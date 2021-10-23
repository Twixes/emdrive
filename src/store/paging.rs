use std::fmt::Debug;

use crate::construct::components::TableDefinition;

use super::encoding::*;

/// Each page is 8 KiB long.
const PAGE_SIZE: usize = 8 * 1024;

/// Latest version of disk data layout. Useful for determining layout compatibility.
const LATEST_LAYOUT_VERSION: u8 = 0;

pub fn empty_page_blob() -> WriteBlob {
    vec![0; PAGE_SIZE]
}

pub fn construct_blank_table() -> WriteBlob {
    // 2 pages, as that's the minimum number of them - 1. the meta page, 2. B+ tree root page (a leaf initially)
    let mut core_blob: WriteBlob = Vec::with_capacity(PAGE_SIZE * 2);
    core_blob.append(
        &mut Page::Meta {
            layout_version: LATEST_LAYOUT_VERSION,
            b_tree_root_page_index: 1,
        }
        .into(),
    );
    core_blob.append(
        &mut Page::BTreeLeaf {
            next_leaf_page_index: 0,
            rows: Vec::new(),
        }
        .into(),
    );
    assert_eq!(core_blob.len(), PAGE_SIZE * 2);
    core_blob
}

/// Possible core page types.
#[derive(Debug, PartialEq, Eq)]
enum Page {
    /// The initial page containing directions for the whole `data` file.
    Meta {
        /// Version of disk data layout that is in use.
        layout_version: u8,
        /// Page index of the B+ tree root. This is the single leaf when tree height is 1, after that it's a node.
        b_tree_root_page_index: PageIndex,
    },
    /// B+ tree node.
    BTreeNode,
    /// B+ tree leaf.
    BTreeLeaf {
        /// Page index of the next leaf in order. 0 means that there's no next leaf, as 0 points to the meta page.
        next_leaf_page_index: PageIndex,
        rows: Vec<Row>,
    },
}

impl From<Page> for WriteBlob {
    fn from(page: Page) -> WriteBlob {
        let mut page_blob: WriteBlob = empty_page_blob();
        match page {
            Page::Meta {
                layout_version,
                b_tree_root_page_index,
            } => {
                let position = 0x00u8.encode(&mut page_blob, 0); // Page type marker
                let position = layout_version.encode(&mut page_blob, position);
                let _final_position = b_tree_root_page_index.encode(&mut page_blob, position);
            }
            Page::BTreeNode => {
                0x20u8.encode(&mut page_blob, 0); // Page type marker
                                                  // TODO
            }
            Page::BTreeLeaf {
                next_leaf_page_index,
                rows,
            } => {
                let position = 0x21u8.encode(&mut page_blob, 0); // Page type marker
                let position = next_leaf_page_index.encode(&mut page_blob, position);
                let mut position = LocalCount::try_from(rows.len())
                    .unwrap()
                    .encode(&mut page_blob, position);
                // Position for writing rows, which is done starting with the back of the blob
                let mut position_back = PAGE_SIZE;
                for row in rows {
                    position_back = row.encode_back(&mut page_blob, position_back);
                    position = LocalCount::try_from(position_back)
                        .unwrap()
                        .encode(&mut page_blob, position);
                }
            }
        };
        assert_eq!(page_blob.len(), PAGE_SIZE, "Page serialization fault - ended up with a blob that is {} B long, instead of the correct {} B", page_blob.len(), PAGE_SIZE);
        page_blob
    }
}

impl<'b> EncodableWithAssumption<'b> for Page {
    type Assumption = TableDefinition;

    fn try_decode_assume(
        blob: ReadBlob<'b>,
        assumption: Self::Assumption,
    ) -> Result<(Self, ReadBlob<'b>), String> {
        let next_page = &blob[PAGE_SIZE..];
        // Select deserialization mode based on page type marker
        match blob[0] {
            // Meta
            0x00 => {
                let (layout_version, rest) = u8::try_decode(&blob[1..])?;
                let (b_tree_root_page_index, _final_rest) = PageIndex::try_decode(rest)?;
                Ok((
                    Self::Meta {
                        layout_version,
                        b_tree_root_page_index,
                    },
                    next_page,
                ))
            }
            // BTreeNode
            0x20 => {
                // TODO
                // - page type (1 byte, `u8`) - `0x20`.
                // - row count (8 bytes, `u64`) - Number of rows contained by all child leaves of the node. A `COUNT` and `OFFSET` optimization.
                // - fan-out (2 bytes, `u16`) - Number of keys in this node.
                // - child page indexes ((fan-out + 1) * 4 bytes, `u32`) - Child pointers.
                // - keys (fan-out of them) - Key values.
                Ok((Self::BTreeNode, next_page))
            }
            // BTreeLeaf
            0x21 => {
                let (next_leaf_page_index, rest) = PageIndex::try_decode(&blob[1..])?;
                let (row_count, rest) = LocalCount::try_decode(rest)?;
                let mut rest = rest;
                let mut rows: Vec<Row> = Vec::with_capacity(row_count as usize);
                let row_data_types: Vec<_> = assumption
                    .columns
                    .iter()
                    .map(|column| &column.data_type)
                    .collect();
                for _ in 0..(row_count as usize) {
                    let (row_address, iteration_rest) = LocalCount::try_decode(rest)?;
                    assert!(row_address > 6, "Row address is {}, but it must be higher than 6, as the first 7 bytes of the page are metadata.", row_address);
                    rest = iteration_rest;
                    let (row, _iteration_rest_back) =
                        Row::try_decode_assume(&blob[row_address as usize..], &row_data_types)?;
                    rows.push(row);
                }
                Ok((
                    Self::BTreeLeaf {
                        next_leaf_page_index,
                        rows,
                    },
                    next_page,
                ))
            }
            _ => Err(format!(
                "Invalid page type marker byte {:#04x} - recognized values are: 0x00, 0x20, 0x21",
                blob[0]
            )),
        }
    }
}

#[cfg(test)]
mod core_serialization_tests {
    use super::*;
    use crate::construct::components::{DataInstance, DataInstanceRaw};
    use crate::store::system::SystemTable;
    use pretty_assertions::assert_eq;
    use std::mem;

    #[test]
    fn blank_table_de_serialization_works() {
        let blank_table_blob: WriteBlob = construct_blank_table();
        let (page_0, rest) =
            Page::try_decode_assume(&blank_table_blob, SystemTable::Tables.get_definition())
                .unwrap();
        assert_eq!(
            page_0,
            Page::Meta {
                layout_version: LATEST_LAYOUT_VERSION,
                b_tree_root_page_index: 1
            }
        );
        let (page_1, _rest) =
            Page::try_decode_assume(rest, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            page_1,
            Page::BTreeLeaf {
                next_leaf_page_index: 0,
                rows: Vec::new()
            }
        );
    }

    #[test]
    fn empty_leaf_de_serialization_works() {
        let leaf_blob: WriteBlob = Page::BTreeLeaf {
            next_leaf_page_index: 0,
            rows: Vec::new(),
        }
        .into();
        let (leaf_page, _rest) =
            Page::try_decode_assume(&leaf_blob, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            leaf_page,
            Page::BTreeLeaf {
                next_leaf_page_index: 0,
                rows: Vec::new()
            }
        );
    }

    #[test]
    fn single_row_de_serialization_works() {
        let leaf_blob: WriteBlob = Page::BTreeLeaf {
            next_leaf_page_index: 0,
            rows: vec![Row(vec![
                DataInstance::Direct(DataInstanceRaw::Uuid(2)),
                DataInstance::Direct(DataInstanceRaw::String("xyz".into())),
            ])],
        }
        .into();
        let (leaf_page, _rest) =
            Page::try_decode_assume(&leaf_blob, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            leaf_page,
            Page::BTreeLeaf {
                next_leaf_page_index: 0,
                rows: vec![Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(2)),
                    DataInstance::Direct(DataInstanceRaw::String("xyz".into()))
                ])]
            }
        );
    }

    #[test]
    fn triple_row_de_serialization_works() {
        let leaf_blob: WriteBlob = Page::BTreeLeaf {
            next_leaf_page_index: 99,
            rows: vec![
                Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(9798799999999)),
                    DataInstance::Direct(DataInstanceRaw::String("Foo 🧐".into())),
                ]),
                Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(0)),
                    DataInstance::Direct(DataInstanceRaw::String("Здравствуйте".into())),
                ]),
                Row(vec![
                    DataInstance::Direct(DataInstanceRaw::Uuid(7)),
                    DataInstance::Direct(DataInstanceRaw::String("".into())),
                ]),
            ],
        }
        .into();
        let (leaf_page, _rest) =
            Page::try_decode_assume(&leaf_blob, SystemTable::Tables.get_definition()).unwrap();
        assert_eq!(
            leaf_page,
            Page::BTreeLeaf {
                next_leaf_page_index: 99,
                rows: vec![
                    Row(vec![
                        DataInstance::Direct(DataInstanceRaw::Uuid(9798799999999)),
                        DataInstance::Direct(DataInstanceRaw::String("Foo 🧐".into())),
                    ]),
                    Row(vec![
                        DataInstance::Direct(DataInstanceRaw::Uuid(0)),
                        DataInstance::Direct(DataInstanceRaw::String("Здравствуйте".into())),
                    ]),
                    Row(vec![
                        DataInstance::Direct(DataInstanceRaw::Uuid(7)),
                        DataInstance::Direct(DataInstanceRaw::String("".into())),
                    ])
                ],
            }
        );
    }

    #[test]
    fn string_de_serialization_works() {
        let sample: &str = "Uśmiech! 😋";
        let mut blob = empty_page_blob();
        sample.to_string().encode(&mut blob, 0);
        let (decoded_smile, rest) = String::try_decode(&blob).unwrap();
        assert_eq!(decoded_smile, sample.to_string());
        assert_eq!(
            rest.len(),
            PAGE_SIZE - mem::size_of::<VarLen>() - sample.len()
        );
    }
}

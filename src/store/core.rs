use crate::construct::components::TableDefinition;
use std::convert;

/// Each page is 8 KiB long.
const PAGE_SIZE: usize = 8 * 1024;

/// Latest version of disk data layout. Useful for determining layout compatibility.
const LATEST_LAYOUT_VERSION: u8 = 0;

pub fn construct_blank_table() -> Vec<u8> {
    // 2 pages, as that's the minimum number of them – 1. the meta page, 2. B+ tree root page (a leaf initially)
    let mut core_blob: Vec<u8> = Vec::with_capacity(PAGE_SIZE * 2);
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
            row_count: 0,
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
        b_tree_root_page_index: u32,
    },
    /// B+ tree node.
    BTreeNode,
    /// B+ tree leaf.
    BTreeLeaf {
        /// Page index of the next leaf in order. 0 means that there's no next leaf, as 0 points to the meta page.
        next_leaf_page_index: u32,
        /// Number of rows stored by this leaf.
        row_count: u16, // TODO: rows: Vec<Row>
    },
}

impl convert::Into<Vec<u8>> for Page {
    fn into(self) -> Vec<u8> {
        let mut page_blob: Vec<u8> = vec![0; PAGE_SIZE];
        match self {
            Self::Meta {
                layout_version,
                b_tree_root_page_index,
            } => {
                page_blob[0] = 0x00; // Page type marker
                page_blob[1] = layout_version;
                page_blob.splice(2..6, b_tree_root_page_index.to_be_bytes());
            }
            Self::BTreeNode => {
                page_blob[0] = 0x20; // Page type marker
                                     // TODO
            }
            Self::BTreeLeaf {
                next_leaf_page_index,
                row_count,
            } => {
                page_blob[0] = 0x21; // Page type marker
                page_blob.splice(1..5, next_leaf_page_index.to_be_bytes());
                page_blob.splice(6..8, row_count.to_be_bytes());
            }
        };
        assert_eq!(page_blob.len(), PAGE_SIZE);
        page_blob
    }
}

impl convert::TryFrom<&[u8]> for Page {
    type Error = String;

    fn try_from(blob: &[u8]) -> Result<Self, Self::Error> {
        if blob.len() != PAGE_SIZE {
            return Err(format!(
                "Received a {} B long page - proper page size is {} B",
                blob.len(),
                PAGE_SIZE
            ));
        }
        // Select deserialization mode based on page type marker
        match blob[0] {
            // Meta
            0x00 => {
                let layout_version = blob[1];
                let b_tree_root_page_index = extract_u32_at(&blob, 2);
                Ok(Self::Meta {
                    layout_version,
                    b_tree_root_page_index
                })
            },
            // BTreeNode
            0x20 => {
                // TODO
                Ok(Self::BTreeNode)
            },
            // BTreeLeaf
            0x21 => {
                let next_leaf_page_index = extract_u32_at(&blob, 1);
                let row_count = extract_u16_at(&blob, 5);
                Ok(Self::BTreeLeaf {
                    next_leaf_page_index,
                    row_count
                })
            },
            _ => Err(format!("Page type marker byte {:#04x} is invalid - recognized values are: 0x00, 0x20, 0x21", blob[0]))
        }
    }
}

fn extract_u16_at(blob: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([blob[index], blob[index + 1]])
}

fn extract_u32_at(blob: &[u8], index: usize) -> u32 {
    u32::from_be_bytes([
        blob[index],
        blob[index + 1],
        blob[index + 2],
        blob[index + 3],
    ])
}

#[cfg(test)]
mod core_serialization_tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::convert::TryFrom;

    #[test]
    fn returns_ok() {
        let blank_table_blob: Vec<u8> = construct_blank_table().into();
        assert_eq!(
            Page::try_from(&blank_table_blob[..PAGE_SIZE]),
            Ok(Page::Meta {
                layout_version: LATEST_LAYOUT_VERSION,
                b_tree_root_page_index: 1
            })
        );
        assert_eq!(
            Page::try_from(&blank_table_blob[PAGE_SIZE..]),
            Ok(Page::BTreeLeaf {
                next_leaf_page_index: 0,
                row_count: 0
            })
        );
    }
}

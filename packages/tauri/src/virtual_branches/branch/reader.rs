use std::path;

use crate::reader::{self, Reader, SubReader};

use super::Branch;

pub struct BranchReader<'reader> {
    reader: &'reader dyn reader::Reader,
}

impl<'reader> BranchReader<'reader> {
    pub fn new(reader: &'reader dyn Reader) -> Self {
        Self { reader }
    }

    pub fn reader(&self) -> &dyn reader::Reader {
        self.reader
    }

    pub fn read(&self, id: &str) -> Result<Branch, reader::Error> {
        if !self
            .reader
            .exists(&path::PathBuf::from(format!("branches/{}", id)))
        {
            return Err(reader::Error::NotFound);
        }

        let single_reader: &dyn crate::reader::Reader =
            &SubReader::new(self.reader, &format!("branches/{}", id));
        Branch::try_from(single_reader)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        sessions,
        test_utils::{Case, Suite},
        virtual_branches::branch::Ownership,
    };

    use super::{super::Writer, *};

    static mut TEST_INDEX: usize = 0;

    fn test_branch() -> Branch {
        unsafe {
            TEST_INDEX += 1;
        }
        Branch {
            id: format!("branch_{}", unsafe { TEST_INDEX }),
            name: format!("branch_name_{}", unsafe { TEST_INDEX }),
            notes: "".to_string(),
            applied: true,
            order: unsafe { TEST_INDEX },
            upstream: Some(
                format!("refs/remotes/origin/upstream_{}", unsafe { TEST_INDEX })
                    .parse()
                    .unwrap(),
            ),
            upstream_head: Some(
                format!("0123456789abcdef0123456789abcdef0123456{}", unsafe {
                    TEST_INDEX
                })
                .parse()
                .unwrap(),
            ),
            created_timestamp_ms: unsafe { TEST_INDEX } as u128,
            updated_timestamp_ms: unsafe { TEST_INDEX + 100 } as u128,
            head: format!("0123456789abcdef0123456789abcdef0123456{}", unsafe {
                TEST_INDEX
            })
            .parse()
            .unwrap(),
            tree: format!("0123456789abcdef0123456789abcdef012345{}", unsafe {
                TEST_INDEX + 10
            })
            .parse()
            .unwrap(),
            ownership: Ownership {
                files: vec![format!("file/{}:1-2", unsafe { TEST_INDEX })
                    .parse()
                    .unwrap()],
            },
        }
    }

    #[test]
    fn test_read_not_found() -> Result<()> {
        let Case { gb_repository, .. } = Suite::default().new_case();

        let session = gb_repository.get_or_create_current_session()?;
        let session_reader = sessions::Reader::open(&gb_repository, &session)?;

        let reader = BranchReader::new(&session_reader);
        let result = reader.read("not found");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "file not found");

        Ok(())
    }

    #[test]
    fn test_read_override() -> Result<()> {
        let Case { gb_repository, .. } = Suite::default().new_case();

        let branch = test_branch();

        let writer = Writer::new(&gb_repository);
        writer.write(&branch)?;

        let session = gb_repository.get_current_session()?.unwrap();
        let session_reader = sessions::Reader::open(&gb_repository, &session)?;

        let reader = BranchReader::new(&session_reader);

        assert_eq!(branch, reader.read(&branch.id).unwrap());

        Ok(())
    }
}

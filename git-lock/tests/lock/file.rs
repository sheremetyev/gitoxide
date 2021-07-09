mod close {

    use git_lock::acquire::Fail;
    use std::io::Write;

    #[test]
    fn acquire_close_commit_to_existing() -> crate::Result {
        let dir = tempfile::tempdir()?;
        let resource = dir.path().join("resource-existing.ext");
        std::fs::write(&resource, b"old state")?;
        let resource_lock = resource.with_extension("ext.lock");
        let mut file = git_lock::File::acquire_to_update_resource(&resource, Fail::Immediately, None)?;
        assert!(resource_lock.is_file());
        file.with_mut(|out| out.write_all(b"hello world"))?;
        let mark = file.close()?;
        assert_eq!(resource_lock, mark.lock_path());
        assert_eq!(mark.commit()?, resource, "returned and initial resource path match");
        assert_eq!(
            std::fs::read(resource)?,
            &b"hello world"[..],
            "it created the resource and wrote the data"
        );
        assert!(!resource_lock.is_file());
        Ok(())
    }
}

mod acquire {
    use git_lock::acquire;
    use std::io::{ErrorKind, Write};

    fn fail_immediately() -> git_lock::acquire::Fail {
        acquire::Fail::Immediately
    }

    #[test]
    fn lock_create_dir_write_commit() -> crate::Result {
        let dir = tempfile::tempdir()?;
        let resource = dir.path().join("a").join("resource-nonexisting");
        let resource_lock = resource.with_extension("lock");
        let mut file =
            git_lock::File::acquire_to_update_resource(&resource, fail_immediately(), Some(dir.path().into()))?;
        assert_eq!(resource_lock, file.lock_path());
        assert!(resource_lock.is_file());
        file.with_mut(|out| out.write_all(b"hello world"))?;
        assert_eq!(file.commit()?, resource, "returned and computed resource path match");
        assert_eq!(
            std::fs::read(resource)?,
            &b"hello world"[..],
            "it created the resource and wrote the data"
        );
        assert!(!resource_lock.is_file());
        Ok(())
    }

    #[test]
    fn lock_write_drop() -> crate::Result {
        let dir = tempfile::tempdir()?;
        let resource = dir.path().join("resource-nonexisting.ext");
        {
            let mut file = git_lock::File::acquire_to_update_resource(&resource, fail_immediately(), None)?;
            file.with_mut(|out| out.write_all(b"probably we will be interrupted"))?;
        }
        assert!(!resource.is_file(), "the file wasn't created");
        Ok(())
    }

    #[test]
    fn lock_non_existing_dir_fails() -> crate::Result {
        let dir = tempfile::tempdir()?;
        let resource = dir.path().join("a").join("resource.ext");
        let res = git_lock::File::acquire_to_update_resource(&resource, fail_immediately(), None);
        assert!(matches!(res, Err(acquire::Error::Io(err)) if err.kind() == ErrorKind::NotFound));
        assert!(dir.path().is_dir(), "it won't meddle with the containing directory");
        assert!(!resource.is_file(), "the resource is not created");
        assert!(
            !resource.parent().unwrap().is_dir(),
            "parent dire wasn't created either"
        );
        Ok(())
    }
}

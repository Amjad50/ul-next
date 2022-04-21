use crate::string::UlString;

pub struct Session {
    internal: ul_sys::ULSession,
    need_to_destroy: bool,

    is_persistent: bool,
    name: String,
    id: u64,
    disk_path: String,
}

impl Session {
    pub(crate) unsafe fn create(
        renderer: ul_sys::ULRenderer,
        is_persistent: bool,
        name: &str,
    ) -> Self {
        let ul_string_name = UlString::from_str(name);
        let internal = ul_sys::ulCreateSession(renderer, is_persistent, ul_string_name.to_ul());

        let id = ul_sys::ulSessionGetId(internal);
        let disk_path = UlString::copy_raw_to_string(ul_sys::ulSessionGetDiskPath(internal));

        Self {
            internal,
            need_to_destroy: true,

            is_persistent,
            name: name.to_string(),
            id,
            disk_path,
        }
    }

    pub(crate) unsafe fn from_raw(raw: ul_sys::ULSession) -> Self {
        let id = ul_sys::ulSessionGetId(raw);
        let disk_path = UlString::copy_raw_to_string(ul_sys::ulSessionGetDiskPath(raw));
        let name = UlString::copy_raw_to_string(ul_sys::ulSessionGetName(raw));
        let is_persistent = ul_sys::ulSessionIsPersistent(raw);

        Self {
            internal: raw,
            need_to_destroy: false,

            is_persistent,
            name,
            id,
            disk_path,
        }
    }

    pub(crate) unsafe fn to_ul(&self) -> ul_sys::ULSession {
        self.internal
    }
}

impl Session {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn disk_path(&self) -> &str {
        &self.disk_path
    }

    pub fn is_persistent(&self) -> bool {
        self.is_persistent
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.need_to_destroy {
            unsafe {
                ul_sys::ulDestroySession(self.internal);
            }
        }
    }
}

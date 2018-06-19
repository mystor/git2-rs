use std::ffi::{CStr, CString};
use std::ptr;

use {raw, Error, Repository, Signature};
use util::Binding;

/// A Mailmap is used to represent a mapping from stored names and emails to
/// real names and emails. Mailmaps can be used to clean up signatures and blame
/// output.
pub struct Mailmap {
    raw: *mut raw::git_mailmap,
}

// Mailmap objects hold exclusive ownership over their state, and thus implement
// send and sync.
unsafe impl Send for Mailmap {}
unsafe impl Sync for Mailmap {}

impl Mailmap {
    /// Allocate a new mailmap object.
    ///
    /// This object is empty, so you'll have to add a mailmap file before you
    /// can do anything with it.
    pub fn new() -> Result<Mailmap, Error> {
        ::init();
        let mut ret = ptr::null_mut();
        unsafe {
            try_call!(raw::git_mailmap_new(&mut ret));
            Ok(Binding::from_raw(ret))
        }
    }

    /// Create a new mailmap instance containing a single mailmap file.
    pub fn from_buffer(buffer: &[u8]) -> Result<Mailmap, Error> {
        ::init();
        let mut ret = ptr::null_mut();
        unsafe {
            try_call!(raw::git_mailmap_from_buffer(&mut ret,
                                                   buffer.as_ptr() as *const _,
                                                   buffer.len()));
            Ok(Binding::from_raw(ret))
        }
    }

    /// Create a new mailmap instance from a repository, loading mailmap files based
    /// on the repository's configuration.
    ///
    /// Mailmaps are loaded in the following order:
    ///  1. '.mailmap' in the root of the repository's working directory, if present.
    ///  2. The blob object identified by the 'mailmap.blob' config entry, if set.
    ///     [NOTE: 'mailmap.blob' defaults to 'HEAD:.mailmap' in bare repositories]
    ///  3. The path in the 'mailmap.file' config entry, if set.
    pub fn from_repository(repo: &Repository) -> Result<Mailmap, Error> {
        let mut ret = ptr::null_mut();
        unsafe {
            try_call!(raw::git_mailmap_from_repository(&mut ret, repo.raw()));
            Ok(Binding::from_raw(ret))
        }
    }

    /// Add a single entry to the given mailmap object. If the entry already
    /// exists, it will be replaced with the new entry.
    pub fn add_entry(&mut self,
                     real_name: Option<&str>, real_email: Option<&str>,
                     replace_name: Option<&str>, replace_email: &str)
                     -> Result<(), Error> {
        let real_name = ::opt_cstr(real_name)?;
        let real_email = ::opt_cstr(real_email)?;
        let replace_name = ::opt_cstr(replace_name)?;
        let replace_email = CString::new(replace_email)?;

        unsafe {
            try_call!(raw::git_mailmap_add_entry(self.raw(),
                                                 real_name,
                                                 real_email,
                                                 replace_name,
                                                 replace_email));
        }
        Ok(())
    }

    /// Resolve a name and email to the corresponding real name and email.
    pub fn resolve<'a>(&'a self, mut name: &'a str, mut email: &'a str)
                       -> Result<(&'a str, &'a str), Error> {
        let cname = CString::new(name)?;
        let cemail = CString::new(email)?;

        let mut rname = ptr::null();
        let mut remail = ptr::null();
        unsafe {
            try_call!(raw::git_mailmap_resolve(&mut rname, &mut remail,
                                               self.raw(), cname, cemail));

            // Compare our pointers, if we got out the same pointers we put in,
            // we need to return our original parameters, otherwise we need to
            // return the strings passed back to us.
            //
            // This trickery is necessary because of how resolve is implemented.
            if rname != cname.as_ptr() {
                name = CStr::from_ptr(rname).to_str()?;
            }
            if remail != cemail.as_ptr() {
                email = CStr::from_ptr(remail).to_str()?;
            }
        }
        Ok((name, email))
    }

    /// Resolve a signature to use real names and emails with a mailmap.
    pub fn resolve_signature(&self, sig: &Signature)
                             -> Result<Signature<'static>, Error> {
        let mut raw = ptr::null_mut();
        unsafe {
            try_call!(raw::git_mailmap_resolve_signature(&mut raw, self.raw(),
                                                         sig.raw()));
            Ok(Binding::from_raw(raw))
        }
    }
}

impl Binding for Mailmap {
    type Raw = *mut raw::git_mailmap;
    unsafe fn from_raw(ptr: *mut raw::git_mailmap) -> Mailmap {
        Mailmap { raw: ptr }
    }
    fn raw(&self) -> *mut raw::git_mailmap { self.raw }
}

impl Drop for Mailmap {
    fn drop(&mut self) {
        unsafe { raw::git_mailmap_free(self.raw) }
    }
}

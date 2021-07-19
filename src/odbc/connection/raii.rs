use super::{ffi, safe, Handle, OdbcObject, Return};
use std::ptr::null_mut;
use super::{DiagnosticRecord, GetDiagRec};
use super::log::*;

/// Wrapper around handle types which ensures the wrapped value is always valid.
///
/// Resource Acquisition Is Initialization
#[derive(Debug)]
pub struct Raii<T: OdbcObject> {
    //Invariant: Should always point to a valid odbc Object
    handle: *mut T,
    // we use phantom data to tell the borrow checker that we need to keep the data source alive
    // for the lifetime of the handle
    //parent: PhantomData<&'p ()>,
}

impl<T: OdbcObject> Handle for Raii<T> {
    type To = T;
    unsafe fn handle(&self) -> *mut T {
        self.handle
    }
}

unsafe impl<T: OdbcObject> safe::Handle for Raii<T> {
    const HANDLE_TYPE: ffi::HandleType = T::HANDLE_TYPE;

    fn handle(&self) -> ffi::SQLHANDLE {        
        self.handle as ffi::SQLHANDLE
    }
}

impl<T: OdbcObject> Drop for Raii<T> {
    fn drop(&mut self) {

        if T::HANDLE_TYPE == ffi::SQL_HANDLE_STMT{
            println!("statment drop");
        }
        if T::HANDLE_TYPE == ffi::SQL_HANDLE_DBC{
            println!("DBC drop");
        }
        if T::HANDLE_TYPE == ffi::SQL_HANDLE_ENV{
            println!("ENV drop");
        }

        let ret = unsafe { ffi::SQLFreeHandle(T::HANDLE_TYPE, self.handle() as ffi::SQLHANDLE) };
        match ret {
            ffi::SQL_SUCCESS => (),
            ffi::SQL_ERROR => {
                let rec = self.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty);
                error!("Error freeing handle: {}", rec)
            },
            _ =>{ 
                let rec = self.get_diag_rec(1).unwrap_or_else(DiagnosticRecord::empty);
                error!("Error freeing handle: {}", rec);
                panic!("Unexepected return value of SQLFreeHandle")},
        }
    }
}

impl<T: OdbcObject> Raii<T> {
    pub fn with_parent<P>(parent: &P) -> Return<Self>
    where
        P: Handle<To = T::Parent>,
    {      
        let mut handle: ffi::SQLHANDLE = null_mut();
        match unsafe {            
            
            ffi::SQLAllocHandle(
                T::HANDLE_TYPE,
                parent.handle() as ffi::SQLHANDLE,
                &mut handle as *mut ffi::SQLHANDLE,
            )
        } { 

            ffi::SQL_SUCCESS => {
                Return::Success(Raii {
                    handle: handle as *mut T,
                    //parent: PhantomData,
                })
            },
            ffi::SQL_SUCCESS_WITH_INFO => Return::SuccessWithInfo(Raii {
                handle: handle as *mut T,
                //parent: PhantomData,
            }),
            ffi::SQL_ERROR => Return::Error,
            _ => panic!("SQLAllocHandle returned unexpected result"),
        }
    }
}


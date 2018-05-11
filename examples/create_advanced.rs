extern crate shared_memory;
use shared_memory::*;
use std::path::PathBuf;

struct SomeState {
    num_listenners: u32,
    message: [u8; 256],
}
//WARNING : Only do this if you know what you're doing.
unsafe impl SharedMemCast for SomeState {}

fn main() -> Result<(), Box<std::error::Error>> {

    //Create a custom configuration for our mapping
    let my_conf = SharedMemConf::new()
        .set_link(&PathBuf::from("shared_mem.link"))
        .set_os_path("test_mapping")
        .set_size(4096)
        .add_lock(LockType::Mutex, 0, 2048)?
        .add_lock(LockType::Mutex, 2047, 2048)?;

    //Create mapping based of our config
    let mut my_shmem = my_conf.create()?;

    println!("Created link file with info : {}", my_shmem);

    {
        //Cast our shared memory as a mutable SomeState struct
        let mut shared_state: WriteLockGuard<SomeState> = match my_shmem.wlock(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };

        //Initialize the memory with default values
        shared_state.num_listenners = 0;
        let src = b"Welcome, we currently have 0 listenners !\x00";
        shared_state.message[0..src.len()].copy_from_slice(src);

        println!("Holding lock for 5 seconds !");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    println!("Waiting for a listenner to connect !");
    //Loop until our memory has changed
    loop {

        //Acquire read lock
        let shared_state: ReadLockGuard<SomeState> = match my_shmem.rlock(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire read lock !"),
        };

        //Check shared memory
        if shared_state.num_listenners > 0 {
            println!("We have a listenner !");
            break;
        }

        //Release the lock before sleeping
        drop(shared_state);
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    //Modify the shared memory just for fun
    {
        let mut shared_state: WriteLockGuard<SomeState> = match my_shmem.wlock(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };

        let src = format!("Goodbye {} listenner(s) !\x00", shared_state.num_listenners);
        shared_state.message[0..src.len()].copy_from_slice(&src.as_bytes());
    }
    Ok(())
}
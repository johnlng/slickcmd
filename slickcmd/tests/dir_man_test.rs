use slickcmd::dir_man::{CurDir, RecentDirs};
use std::cell::RefCell;
use std::rc::Rc;

mod common;

#[test]
fn test_cur_dir() {
    let mut cur_dir = CurDir::default();

    let set_dirs = Rc::new(RefCell::new(Vec::<String>::new()));

    let set_dirs2 = set_dirs.clone();
    cur_dir.on_set = Some(Box::new(move |dir| {
        set_dirs2.borrow_mut().push(dir.into());
    }));

    cur_dir.set(r"C:\Users\John\");
    assert_eq!(cur_dir.get(), r"C:\Users\John\");

    cur_dir.go_up();
    assert_eq!(cur_dir.get(), r"C:\Users\");

    assert_eq!(cur_dir._inspect(), vec![r"C:\Users\John\", r"C:\Users\@"]);

    cur_dir.set(r"D:\Test\");
    let dirs = vec![r"C:\Users\John\", r"C:\Users\", r"D:\Test\@"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_back();
    let dirs = vec![r"C:\Users\John\", r"C:\Users\@", r"D:\Test\"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_back();
    let dirs = vec![r"C:\Users\John\@", r"C:\Users\", r"D:\Test\"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_forward();
    let dirs = vec![r"C:\Users\John\", r"C:\Users\@", r"D:\Test\"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_up();
    let dirs = vec![r"C:\Users\John\", r"C:\Users\", r"C:\@"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_up();
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.go_back();
    cur_dir.go_back();
    let dirs = vec![r"C:\Users\John\@", r"C:\Users\", r"C:\"];
    assert_eq!(cur_dir._inspect(), dirs);

    cur_dir.set(r"C:\Windows\");
    let dirs = vec![r"C:\Users\John\", r"C:\Windows\@"];
    assert_eq!(cur_dir._inspect(), dirs);

    let dirs = vec![
        r"C:\Users\John\",
        r"C:\Users\",
        r"D:\Test\",
        r"C:\",
        r"C:\Windows\",
    ];
    assert_eq!(*set_dirs.borrow(), dirs);
}

#[test]
fn test_recent_dirs() {
    let recent_dirs = RecentDirs::default();
    recent_dirs.use_dir(r"C:\Users\John");
    recent_dirs.use_dir(r"C:\Users");
    recent_dirs.use_dir(r"D:\Test");
    recent_dirs.use_dir(r"C:\Users");

    let dirs = vec![r"C:\Users\John", r"D:\Test", r"C:\Users"];
    assert_eq!(recent_dirs._inspect(), dirs);
}

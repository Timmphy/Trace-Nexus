use is_elevated::is_elevated;

pub fn check_admin() -> bool {
    is_elevated()
}

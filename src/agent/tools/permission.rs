#[derive(Debug, PartialEq, Eq)]
pub enum Permission {
    // Always allow, don't ask the user for a permission
    Implicit,
    // Request the user for a permission in manual mode
    // Allow in auto or yolo (usually for edit operations + simple commands)
    AllowAutomatic,
    // Request the user for a permission, in manual or auto mode
    // Allow in yolo (usually for more complex commands)
    RequireApproval,
    // Always reject, even in yolo  mode
    Never,
}

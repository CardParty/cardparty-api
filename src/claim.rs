use crate::ids::Id;

enum Permissions {
    SelfAcess,
    OtherAcess(Id),
    Admin,
}

struct Claim {
    user_id: Id,
    expiration: usize,
    issue_date: usize,
    permissions: Vec<Permissions>,
}

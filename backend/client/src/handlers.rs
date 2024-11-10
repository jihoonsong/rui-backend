pub trait ClientHandlers {
    fn add_member(&self, identity_commitment: String);
}

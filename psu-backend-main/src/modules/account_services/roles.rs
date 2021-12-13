use crate::MainPGDatabase;

pub struct Role {
  id: String,
  name: String,
  desc: String,
  permissions: Vec<String>,
  level: i64
}

pub fn get_roles(conn: &MainPGDatabase) {

}

pub fn set_role(conn: &MainPGDatabase) {
  
}

pub fn modify_role(conn: &MainPGDatabase) {
  
}

pub fn create_role(conn: &MainPGDatabase) {

}
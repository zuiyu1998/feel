pub mod content;
pub mod label;
pub mod user;

pub mod prelude {
    pub use crate::content::entities::prelude::*;
    pub use crate::label::entities::prelude::*;
    pub use crate::user::entities::prelude::*;
}

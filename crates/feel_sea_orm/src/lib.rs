pub mod content;
pub mod follow;
pub mod label;
pub mod user;

pub mod prelude {
    pub use crate::content::entities::prelude::*;
    pub use crate::follow::entities::prelude::*;
    pub use crate::label::entities::prelude::*;
    pub use crate::user::entities::prelude::*;
}

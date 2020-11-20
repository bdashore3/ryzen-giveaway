use serenity::framework::standard::macros::group;

use crate::commands::{general::*, giveaway::*};

#[group]
#[help_available(false)]
#[commands(ping, setup)]
pub struct General;

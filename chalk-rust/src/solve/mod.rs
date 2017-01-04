#![allow(dead_code)]

mod environment;
mod infer;
mod quantify;
mod implemented;

pub struct Solution<G> {
    successful: bool,
    refined_goal: G
}

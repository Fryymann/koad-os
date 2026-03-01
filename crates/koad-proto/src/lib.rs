pub mod kernel {
    tonic::include_proto!("koad.kernel");
}

pub mod skill {
    tonic::include_proto!("koad.skill");
}

pub mod spine {
    pub mod v1 {
        tonic::include_proto!("koad.spine.v1");
    }
}

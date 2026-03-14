pub mod skill {
    tonic::include_proto!("koad.skill");
}

pub mod spine {
    pub mod v1 {
        tonic::include_proto!("koad.spine.v1");
    }
}

pub mod citadel {
    pub mod v5 {
        tonic::include_proto!("koad.citadel.v5");
    }
}

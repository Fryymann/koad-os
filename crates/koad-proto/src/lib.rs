pub mod skill {
    tonic::include_proto!("koad.skill");
}

pub mod citadel {
    pub mod v5 {
        tonic::include_proto!("koad.citadel.v5");
    }
}

pub mod cass {
    pub mod v1 {
        tonic::include_proto!("koad.cass.v1");
    }
}

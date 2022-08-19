use lib_device::*;

#[derive(Clone)]
pub struct CrossBeamChannel {
    pub send: crossbeam_channel::Sender<Vec<Device>>,
    pub receive: crossbeam_channel::Receiver<Vec<Device>>,
}

use lib_device::*;

#[derive(Clone)]
pub struct CrossBeamChannel {
    pub send: crossbeam_channel::Sender<Vec<Device>>,
    pub receive: crossbeam_channel::Receiver<Vec<Device>>,
}

#[derive(Clone)]
pub struct DeviceBeam {
    pub read: Option<CrossBeamChannel>,
    pub update: Option<CrossBeamChannel>,
}

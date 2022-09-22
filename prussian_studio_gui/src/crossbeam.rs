use lib_device::*;

use crate::panels::central_panel::JsonWriteChannel;

#[derive(Clone)]
pub struct CrossBeamChannel {
    pub send: crossbeam_channel::Sender<Vec<Device>>,
    pub receive: crossbeam_channel::Receiver<Vec<Device>>,
}
#[derive(Clone)]
pub struct CrossBeamSocketChannel {
    pub send: crossbeam_channel::Sender<JsonWriteChannel>,
    pub receive: crossbeam_channel::Receiver<JsonWriteChannel>,
}

#[derive(Clone)]
pub struct DeviceBeam {
    pub read: Option<CrossBeamChannel>,
    pub update: Option<CrossBeamChannel>,
}

#[derive(Clone)]
pub struct DeviceMsgBeam {
    pub send: crossbeam_channel::Sender<DeviceMsg>,
    pub receive: crossbeam_channel::Receiver<DeviceMsg>,
}

use std::collections::HashMap;
use wayland_client::{
    backend::ObjectId,
    delegate_noop,
    protocol::{wl_registry::WlRegistry, wl_seat::WlSeat},
    Connection, Dispatch, Proxy,
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{Event as DataControlDeviceEvent, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
};

struct AppState {
    seat: Option<WlSeat>,
    data_control_manager: Option<ZwlrDataControlManagerV1>,
    data_control_device: Option<ZwlrDataControlDeviceV1>,
    offer_mime_types: HashMap<ObjectId, Vec<String>>,
}

impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
        _state: &mut Self,
        _registry: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;

        match event {
            Event::Global {
                name,
                interface,
                version,
            } => {
                match interface.as_str() {
                    "zwlr_data_control_manager_v1" => {
                        let version_to_bind = if version > 2 { 2 } else { version }; // cap at version 2
                        let data_control_manager = _registry
                            .bind::<ZwlrDataControlManagerV1, _, _>(
                                name,
                                version_to_bind,
                                qhandle,
                                (),
                            );
                        _state.data_control_manager = Some(data_control_manager);
                    }
                    "wl_seat" => {
                        let seat = _registry.bind::<WlSeat, _, _>(name, version, qhandle, ());
                        _state.seat = Some(seat);
                    }
                    _ => {}
                }
            }
            Event::GlobalRemove { name: _ } => {}
            _ => {}
        }
    }
}

delegate_noop!(AppState: ignore WlSeat);
delegate_noop!(AppState: ignore ZwlrDataControlManagerV1);

impl wayland_client::Dispatch<ZwlrDataControlDeviceV1, (), AppState> for AppState {
    fn event(
        state: &mut AppState,
        _proxy: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<AppState>,
    ) {
        match event {
            DataControlDeviceEvent::DataOffer { id: offer } => {
                let offer_id = offer.id();
                state.offer_mime_types.insert(offer_id.clone(), Vec::new());
            }
            DataControlDeviceEvent::Selection { id } => {
                // TODO: Handle null case later
                let offer = id.unwrap();
                let offer_id = offer.id();
                println!("📋 Selection: we got selection with id {:?}", offer_id);
                if let Some(mime_types) = state.offer_mime_types.get(&offer_id) {
                    for mime_type in mime_types {
                        println!("📋 {}", mime_type);
                    }
                }
            }
            DataControlDeviceEvent::PrimarySelection { id } => {
                if let Some(offer) = id {
                    let offer_id = offer.id();
                    state.offer_mime_types.remove(&offer_id);
                }
            }
            _ => panic!("Unknown ZwlrDataControlDevice event"),
        }
    }

    // Unusual because zwlr_data_control_device_v1::data_offer event creates child objects
    fn event_created_child(
        opcode: u16,
        qhandle: &wayland_client::QueueHandle<AppState>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        match opcode {
            // Opcode 0 is the data_offer event that creates ZwlrDataControlOfferV1
            0 => qhandle.make_data::<ZwlrDataControlOfferV1, _>(()),
            _ => panic!(
                "Unknown child object opcode {} for ZwlrDataControlDeviceV1",
                opcode
            ),
        }
    }
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: <ZwlrDataControlOfferV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_offer_v1::Event;
        match event {
            Event::Offer { mime_type } => {
                let offer_id = _proxy.id();
                if let Some(mime_types) = state.offer_mime_types.get_mut(&offer_id) {
                    mime_types.push(mime_type.clone());
                }
            }
            _ => {}
        }
    }
}

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // Create a wl_registry object by sending the wl_display.get_registry request
    let _registry = display.get_registry(&qh, ());

    let mut state = AppState {
        data_control_manager: None,
        seat: None,
        data_control_device: None,
        offer_mime_types: HashMap::new(),
    };
    event_queue.blocking_dispatch(&mut state)?;

    if state.data_control_manager.is_none() || state.seat.is_none() {
        return Err("Missing zwlr_data_control_manager_v1 or wl_seat".into());
    }

    let data_control_manager = state.data_control_manager.as_ref().unwrap();
    let seat = state.seat.as_ref().unwrap();

    let data_control_device = data_control_manager.get_data_device(seat, &qh, ());
    state.data_control_device = Some(data_control_device);

    println!("Listening for clipboard events forever...");

    loop {
        match event_queue.blocking_dispatch(&mut state) {
            Ok(_) => {
                // Continue processing events
            }
            Err(e) => {
                eprintln!("Error processing events: {}", e);
                break;
            }
        }
    }

    Ok(())
}

//! Serves a Bluetooth GATT application.
use std::{collections::BTreeMap, time::Duration};

use blurz::{gatt, LeAdvertisement};
use futures::FutureExt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::sleep,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> blurz::Result<()> {
    let service_uuid = "9643735b-c62e-4717-0000-61abaf5abc8e".parse().unwrap();
    let characteristic_uuid = "9643735b-c62e-4717-0001-61abaf5abc8e".parse().unwrap();

    let session = blurz::Session::new().await?;
    let adapter_names = session.adapter_names().await?;
    let adapter_name = adapter_names.first().expect("No Bluetooth adapter present");
    let adapter = session.adapter(&adapter_name)?;

    println!("Advertising on Bluetooth adapter {}: {}", &adapter_name, adapter.address().await?);

    let mut manufacturer_data = BTreeMap::new();
    manufacturer_data.insert(0xffff, vec![0x21, 0x22, 0x23, 102]);

    // let mut service_data = BTreeMap::new();
    // service_data.insert(service_uuid, vec![0x31, 0x32, 0x33, 0x34, 0x35]);

    let le_advertisement = LeAdvertisement {
        service_uuids: vec![service_uuid].into_iter().collect(),
        manufacturer_data,
        // service_data,
        discoverable: Some(true),
        local_name: Some("gatt_server".to_string()),
        ..Default::default()
    };
    let adv_handle = adapter.le_advertise(le_advertisement).await?;

    println!("Serving GATT application on Bluetooth adapter {}", &adapter_name);

    let app = gatt::local::Application {
        services: vec![gatt::local::Service {
            uuid: service_uuid,
            primary: true,
            characteristics: vec![gatt::local::Characteristic {
                uuid: characteristic_uuid,
                read: Some(gatt::local::CharacteristicRead {
                    fun: Box::new(|req| {
                        async move {
                            println!("Read request: {:?}", &req);
                            Ok(vec![1, 2, 3])
                        }
                        .boxed()
                    }),
                    flags: gatt::local::CharacteristicReadFlags { read: true, ..Default::default() },
                }),
                write: Some(gatt::local::CharacteristicWrite {
                    method: gatt::local::CharacteristicWriteMethod::Fn(Box::new(|value, req| {
                        async move {
                            println!("Write request {:?} with value {:?}", &req, &value);
                            Ok(())
                        }
                        .boxed()
                    })),
                    flags: gatt::local::CharacteristicWriteFlags { write: true, ..Default::default() },
                }),
                ..Default::default()
            }],
        }],
    };
    let app_handle = adapter.serve_gatt_application(app).await?;

    println!("Press enter to quit");
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let _ = lines.next_line().await;

    println!("Removing application");
    drop(app_handle);
    drop(adv_handle);

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

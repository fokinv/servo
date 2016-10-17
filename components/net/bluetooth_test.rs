/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_thread::BluetoothManager;
use device::bluetooth::{BluetoothAdapter, BluetoothDevice};
use device::bluetooth::{BluetoothGATTCharacteristic, BluetoothGATTDescriptor, BluetoothGATTService};
use ipc_channel::ipc::IpcSender;
use net_traits::bluetooth_thread::{BluetoothError, BluetoothResult};
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::string::String;
use uuid::Uuid;

thread_local!(pub static CACHED_IDS: RefCell<HashSet<Uuid>> = RefCell::new(HashSet::new()));

const ADAPTER_ERROR: &'static str = "No adapter found";
const FAILED_SET_ERROR: &'static str = "Failed to set an attribute for testing";
const WRONG_DATA_SET_ERROR: &'static str = "Wrong data set name was provided";
const READ_FLAG: &'static str = "read";
const WRITE_FLAG: &'static str = "write";

// Adapter names
const NOT_PRESENT_ADAPTER: &'static str = "NotPresentAdapter";
const NOT_POWERED_ADAPTER: &'static str = "NotPoweredAdapter";
const EMPTY_ADAPTER: &'static str = "EmptyAdapter";
const GLUCOSE_HEART_RATE_ADAPTER: &'static str = "GlucoseHeartRateAdapter";
const UNICODE_DEVICE_ADAPTER: &'static str = "UnicodeDeviceAdapter";
const MISSING_SERVICE_HEART_RATE_ADAPTER: &'static str = "MissingServiceHeartRateAdapter";
const MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER: &'static str = "MissingCharacteristicHeartRateAdapter";
const MISSING_DESCRIPTOR_HEART_RATE_ADAPTER: &'static str = "MissingDescriptorHeartRateAdapter";
const HEART_RATE_ADAPTER: &'static str = "HeartRateAdapter";
const EMPTY_NAME_HEART_RATE_ADAPTER: &'static str = "EmptyNameHeartRateAdapter";
const NO_NAME_HEART_RATE_ADAPTER: &'static str = "NoNameHeartRateAdapter";
const TWO_HEART_RATE_SERVICES_ADAPTER: &'static str = "TwoHeartRateServicesAdapter";
const BLACKLIST_TEST_ADAPTER: &'static str = "BlacklistTestAdapter";

// Device names
const CONNECTABLE_DEVICE_NAME: &'static str = "Connectable Device";
const EMPTY_DEVICE_NAME: &'static str = "";
const GLUCOSE_DEVICE_NAME: &'static str = "Glucose Device";
const HEART_RATE_DEVICE_NAME: &'static str = "Heart Rate Device";
const UNICODE_DEVICE_NAME: &'static str = "❤❤❤❤❤❤❤❤❤";

// Device addresses
const CONNECTABLE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:04";
const GLUCOSE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:01";
const HEART_RATE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:03";
const UNICODE_DEVICE_ADDRESS: &'static str = "00:00:00:00:00:02";

// Service UUIDs
const BLACKLIST_TEST_SERVICE_UUID: &'static str = "611c954a-263b-4f4a-aab6-01ddb953f985";
const DEVICE_INFORMATION_UUID: &'static str = "0000180a-0000-1000-8000-00805f9b34fb";
const GENERIC_ACCESS_SERVICE_UUID: &'static str = "00001800-0000-1000-8000-00805f9b34fb";
const GLUCOSE_SERVICE_UUID: &'static str = "00001808-0000-1000-8000-00805f9b34fb";
const HEART_RATE_SERVICE_UUID: &'static str = "0000180d-0000-1000-8000-00805f9b34fb";
const HUMAN_INTERFACE_DEVICE_SERVICE_UUID: &'static str = "00001812-0000-1000-8000-00805f9b34fb";
const TX_POWER_SERVICE_UUID: &'static str = "00001804-0000-1000-8000-00805f9b34fb";

// Characteristic UUIDs
const BLACKLIST_EXCLUDE_READS_CHARACTERISTIC_UUID: &'static str = "bad1c9a2-9a5b-4015-8b60-1579bbbf2135";
const BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID: &'static str = "00002a38-0000-1000-8000-00805f9b34fb";
const DEVICE_NAME_CHARACTERISTIC_UUID: &'static str = "00002a00-0000-1000-8000-00805f9b34fb";
const HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID: &'static str = "00002a37-0000-1000-8000-00805f9b34fb";
const PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID: &'static str = "00002a02-0000-1000-8000-00805f9b34fb";
const SERIAL_NUMBER_STRING_UUID: &'static str = "00002a25-0000-1000-8000-00805f9b34fb";

// Descriptor UUIDs
const BLACKLIST_EXCLUDE_READS_DESCRIPTOR_UUID: &'static str = "aaaaaaaa-aaaa-1181-0510-810819516110";
const BLACKLIST_DESCRIPTOR_UUID: &'static str = "07711111-6104-0970-7011-1107105110aaa";
const CHARACTERISTIC_USER_DESCRIPTION_UUID: &'static str = "00002901-0000-1000-8000-00805f9b34fb";
const CLIENT_CHARACTERISTIC_CONFIGURATION_UUID: &'static str = "00002902-0000-1000-8000-00805f9b34fb";
const NUMBER_OF_DIGITALS_UUID: &'static str = "00002909-0000-1000-8000-00805f9b34fb";

fn generate_id() -> Uuid {
    let mut id = Uuid::nil();
    let mut generated = false;
    while !generated {
        id = Uuid::new_v4();
        CACHED_IDS.with(|cache|
            if !cache.borrow().contains(&id) {
                cache.borrow_mut().insert(id.clone());
                generated = true;
            }
        );
    }
    id
}

// Set the adapter's name, is_powered and is_discoverable attributes
fn set_adapter(adapter: &BluetoothAdapter, adapter_name: String) -> Result<(), Box<Error>> {
    try!(adapter.set_name(adapter_name));
    try!(adapter.set_powered(true));
    try!(adapter.set_discoverable(true));
    Ok(())
}

// Create Device
fn create_device(adapter: &BluetoothAdapter,
                 name: String,
                 address: String)
                 -> Result<BluetoothDevice, Box<Error>> {
    let device = BluetoothDevice::create_device(adapter.clone(), generate_id().to_string());
    try!(device.set_name(Some(name)));
    try!(device.set_address(address));
    try!(device.set_connectable(true));
    Ok(device)
}

// Create Device with UUIDs
fn create_device_with_uuids(adapter: &BluetoothAdapter,
                            name: String,
                            address: String,
                            uuids: Vec<String>)
                            -> Result<BluetoothDevice, Box<Error>> {
    let device = try!(create_device(adapter, name, address));
    try!(device.set_uuids(uuids));
    Ok(device)
}

// Create Service
fn create_service(device: &BluetoothDevice,
                  uuid: String)
                  -> Result<BluetoothGATTService, Box<Error>>{
    let service = BluetoothGATTService::create_service(device.clone(), generate_id().to_string());
    try!(service.set_uuid(uuid));
    Ok(service)
}

// Create Characteristic
fn create_characteristic(service: &BluetoothGATTService,
                         uuid: String)
                         -> Result<BluetoothGATTCharacteristic, Box<Error>>{
    let characteristic = BluetoothGATTCharacteristic::create_characteristic(service.clone(), generate_id().to_string());
    try!(characteristic.set_uuid(uuid));
    Ok(characteristic)
}

// Create Characteristic with value
fn create_characteristic_with_value(service: &BluetoothGATTService,
                                    uuid: String,
                                    value: Vec<u8>)
                                    -> Result<BluetoothGATTCharacteristic, Box<Error>>{
    let characteristic = try!(create_characteristic(service, uuid));
    try!(characteristic.set_value(value));
    Ok(characteristic)
}

// Create Descriptor
fn create_descriptor(characteristic: &BluetoothGATTCharacteristic,
                                     uuid: String)
                                     -> Result<BluetoothGATTDescriptor, Box<Error>>{
    let descriptor = BluetoothGATTDescriptor::create_descriptor(characteristic.clone(), generate_id().to_string());
    try!(descriptor.set_uuid(uuid));
    Ok(descriptor)
}

// Create Descriptor with value
fn create_descriptor_with_value(characteristic: &BluetoothGATTCharacteristic,
                                uuid: String,
                                value: Vec<u8>)
                                -> Result<BluetoothGATTDescriptor, Box<Error>>{
    let descriptor = try!(create_descriptor(characteristic, uuid));
    try!(descriptor.set_value(value));
    Ok(descriptor)
}

fn create_heart_rate_service(device: &BluetoothDevice)
                             -> Result<BluetoothGATTService, Box<Error>>{
    // Heart Rate Service
    let heart_rate_service = try!(create_service(device,
                                            HEART_RATE_SERVICE_UUID.to_owned()));

    // Heart Rate Measurement Characteristic
    let _heart_rate_measurement_characteristic =
        try!(create_characteristic_with_value(&heart_rate_service,
                                         HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID.to_owned(),
                                         vec![3]));

    // Body Sensor Location Characteristic 1
    let _body_sensor_location_characteristic_1 =
        try!(create_characteristic_with_value(&heart_rate_service,
                                         BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                         vec![1]));
    // Body Sensor Location Characteristic 2
    let _body_sensor_location_characteristic_2 =
        try!(create_characteristic_with_value(&heart_rate_service,
                                         BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                         vec![2]));
    Ok(heart_rate_service)
}

fn create_generic_access_service(device: &BluetoothDevice)
                                 -> Result<BluetoothGATTService, Box<Error>>{
    let generic_access_service = try!(create_service(device,
                                                GENERIC_ACCESS_SERVICE_UUID.to_owned()));
    // Device Name Characteristic
    let device_name_characteristic =
        try!(create_characteristic_with_value(&generic_access_service,
                                         DEVICE_NAME_CHARACTERISTIC_UUID.to_owned(),
                                         HEART_RATE_DEVICE_NAME.as_bytes().to_vec()));
    try!(device_name_characteristic.set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()]));

    // Number of Digitals descriptor
    let _number_of_digitals_descriptor =
        try!(create_descriptor_with_value(&device_name_characteristic,
                                     NUMBER_OF_DIGITALS_UUID.to_owned(),
                                     vec![49, 49]));

    // Characteristic User Description Descriptor
    let _characteristic_user_description =
        try!(create_descriptor_with_value(&device_name_characteristic,
                                     CHARACTERISTIC_USER_DESCRIPTION_UUID.to_owned(),
                                     vec![22, 33, 44, 55]));

    // Client Characteristic Configuration descriptor
    let _client_characteristic_configuration =
        try!(create_descriptor_with_value(&device_name_characteristic,
                                     CLIENT_CHARACTERISTIC_CONFIGURATION_UUID.to_owned(),
                                     vec![0, 0]));

    // Peripheral Privacy Flag Characteristic
    let peripheral_privacy_flag_characteristic =
        try!(create_characteristic(&generic_access_service,
                              PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID.to_owned()));
    try!(peripheral_privacy_flag_characteristic
                                  .set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()]));
    Ok(generic_access_service)
}

// Create Heart Rate Device
fn create_heart_rate_device(adapter: &BluetoothAdapter)
                            -> Result<BluetoothDevice, Box<Error>> {
    // Heart Rate Device
    let heart_rate_device =
        try!(create_device_with_uuids(adapter,
                                 HEART_RATE_DEVICE_NAME.to_owned(),
                                 HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                 vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(), HEART_RATE_SERVICE_UUID.to_owned()]));

    // Generic Access Service
    let _generic_access_service = try!(create_generic_access_service(&heart_rate_device));

    // Heart Rate Service
    let _heart_rate_service = try!(create_heart_rate_service(&heart_rate_device));

    Ok(heart_rate_device)
}

pub fn test(manager: &mut BluetoothManager, data_set_name: String) -> Result<(), Box<Error>> {
    match manager.get_or_create_adapter().as_ref() {
        Some(adapter) => {
            match data_set_name.as_str() {
                NOT_PRESENT_ADAPTER => {
                    try!(set_adapter(adapter, NOT_PRESENT_ADAPTER.to_owned()));
                    try!(adapter.set_present(false));
                },
                NOT_POWERED_ADAPTER => {
                    try!(set_adapter(adapter, NOT_POWERED_ADAPTER.to_owned()));
                    try!(adapter.set_powered(false));
                },
                EMPTY_ADAPTER => {
                    try!(set_adapter(adapter, EMPTY_ADAPTER.to_owned()));
                },
                GLUCOSE_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, GLUCOSE_HEART_RATE_ADAPTER.to_owned()));

                    // Glucose Device
                    let _glucose_device =
                    try!(create_device_with_uuids(adapter,
                                             GLUCOSE_DEVICE_NAME.to_owned(),
                                             GLUCOSE_DEVICE_ADDRESS.to_owned(),
                                             vec![GLUCOSE_SERVICE_UUID.to_owned(), TX_POWER_SERVICE_UUID.to_owned()]));

                    // Heart Rate Device
                    let _heart_rate_device = try!(create_device_with_uuids(adapter,
                                                                      HEART_RATE_DEVICE_NAME.to_owned(),
                                                                      HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                                                      vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                                           HEART_RATE_SERVICE_UUID.to_owned()]));
                },
                UNICODE_DEVICE_ADAPTER => {
                    try!(set_adapter(adapter, UNICODE_DEVICE_ADAPTER.to_owned()));

                    // Unicode Device
                    let _unicode_device = try!(create_device(adapter,
                                                        UNICODE_DEVICE_NAME.to_owned(),
                                                        UNICODE_DEVICE_ADDRESS.to_owned()));
                },
                MISSING_SERVICE_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, MISSING_SERVICE_HEART_RATE_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let _heart_rate_device = try!(create_device_with_uuids(adapter,
                                                                      HEART_RATE_DEVICE_NAME.to_owned(),
                                                                      HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                                                      vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                                           HEART_RATE_SERVICE_UUID.to_owned()]));
                },
                MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, MISSING_CHARACTERISTIC_HEART_RATE_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let heart_rate_device = try!(create_device_with_uuids(adapter,
                                                                     HEART_RATE_DEVICE_NAME.to_owned(),
                                                                     HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                                                     vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                                          HEART_RATE_SERVICE_UUID.to_owned()]));
                    // Generic Access Service
                    let _generic_access_service = try!(create_service(&heart_rate_device,
                                                                 GENERIC_ACCESS_SERVICE_UUID.to_owned()));

                    // Heart Rate Service
                    let _heart_rate_service = try!(create_service(&heart_rate_device,
                                                             HEART_RATE_SERVICE_UUID.to_owned()));
                },
                MISSING_DESCRIPTOR_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, MISSING_DESCRIPTOR_HEART_RATE_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let heart_rate_device = try!(create_device_with_uuids(adapter,
                                                                     HEART_RATE_DEVICE_NAME.to_owned(),
                                                                     HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                                                     vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                                          HEART_RATE_SERVICE_UUID.to_owned()]));
                    // Generic Access Service
                    let generic_access_service = try!(create_service(&heart_rate_device,
                                                                GENERIC_ACCESS_SERVICE_UUID.to_owned()));

                    // Device Name Characteristic
                    let device_name_characteristic =
                        try!(create_characteristic_with_value(&generic_access_service,
                                                         DEVICE_NAME_CHARACTERISTIC_UUID.to_owned(),
                                                         HEART_RATE_DEVICE_NAME.as_bytes().to_vec()));

                    // Peripheral Privacy Flag Characteristic
                    let peripheral_privacy_flag_characteristic =
                        try!(create_characteristic(&generic_access_service,
                                              PERIPHERAL_PRIVACY_FLAG_CHARACTERISTIC_UUID.to_owned()));
                    try!(peripheral_privacy_flag_characteristic
                                                  .set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()]));

                    // Heart Rate Service
                    let _heart_rate_service = try!(create_heart_rate_service(&heart_rate_device));
                },
                HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, HEART_RATE_ADAPTER.to_owned()));
                    // Heart Rate Device
                    let _heart_rate_device = try!(create_heart_rate_device(adapter));
                },
                EMPTY_NAME_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, EMPTY_NAME_HEART_RATE_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let heart_rate_device = try!(create_heart_rate_device(adapter));
                    try!(heart_rate_device.set_name(Some(EMPTY_DEVICE_NAME.to_owned())));
                },
                NO_NAME_HEART_RATE_ADAPTER => {
                    try!(set_adapter(adapter, NO_NAME_HEART_RATE_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let heart_rate_device = try!(create_heart_rate_device(adapter));
                    try!(heart_rate_device.set_name(None));
                },
                TWO_HEART_RATE_SERVICES_ADAPTER => {
                    try!(set_adapter(adapter, TWO_HEART_RATE_SERVICES_ADAPTER.to_owned()));

                    // Heart Rate Device
                    let heart_rate_device = try!(create_device_with_uuids(adapter,
                                                                     HEART_RATE_DEVICE_NAME.to_owned(),
                                                                     HEART_RATE_DEVICE_ADDRESS.to_owned(),
                                                                     vec![GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                                          HEART_RATE_SERVICE_UUID.to_owned()]));
                    try!(heart_rate_device
                        .set_uuids(vec![
                            GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                            HEART_RATE_SERVICE_UUID.to_owned(),
                            HEART_RATE_SERVICE_UUID.to_owned()]));

                    // Generic Access Service
                    let _generic_access_service = try!(create_generic_access_service(&heart_rate_device));

                    // Heart Rate Service
                    let heart_rate_service_1 = try!(create_service(&heart_rate_device,
                                                              HEART_RATE_SERVICE_UUID.to_owned()));

                    // Heart Rate Service
                    let heart_rate_service_2 = try!(create_service(&heart_rate_device,
                                                              HEART_RATE_SERVICE_UUID.to_owned()));

                    // Heart Rate Measurement Characteristic
                    let _heart_rate_measurement_characteristic =
                        try!(create_characteristic_with_value(&heart_rate_service_1,
                                                         HEART_RATE_MEASUREMENT_CHARACTERISTIC_UUID.to_owned(),
                                                         vec![3]));

                    // Body Sensor Location Characteristic 1
                    let _body_sensor_location_characteristic_1 =
                        try!(create_characteristic_with_value(&heart_rate_service_1,
                                                         BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                                         vec![1]));
                    // Body Sensor Location Characteristic 2
                    let _body_sensor_location_characteristic_2 =
                        try!(create_characteristic_with_value(&heart_rate_service_2,
                                                         BODY_SENSOR_LOCATION_CHARACTERISTIC_UUID.to_owned(),
                                                         vec![2]));
                },
                BLACKLIST_TEST_ADAPTER => {
                    try!(set_adapter(adapter, BLACKLIST_TEST_ADAPTER.to_owned()));

                    // Connectable Device
                    let connectable_device =
                        try!(create_device_with_uuids(adapter,
                                                 CONNECTABLE_DEVICE_NAME.to_owned(),
                                                 CONNECTABLE_DEVICE_ADDRESS.to_owned(),
                                                 vec![BLACKLIST_TEST_SERVICE_UUID.to_owned(),
                                                      DEVICE_INFORMATION_UUID.to_owned(),
                                                      GENERIC_ACCESS_SERVICE_UUID.to_owned(),
                                                      HEART_RATE_SERVICE_UUID.to_owned(),
                                                      HUMAN_INTERFACE_DEVICE_SERVICE_UUID.to_owned()]));

                    // Blacklist Test Service
                    let blacklist_test_service =
                        try!(create_service(&connectable_device,
                                       BLACKLIST_TEST_SERVICE_UUID.to_owned()));

                    // Blacklist Exclude Reads Characteristic
                    let blacklist_exclude_reads_characteristic =
                        try!(create_characteristic(&blacklist_test_service,
                                              BLACKLIST_EXCLUDE_READS_CHARACTERISTIC_UUID.to_owned()));
                    try!(blacklist_exclude_reads_characteristic
                                                  .set_flags(vec![READ_FLAG.to_string(), WRITE_FLAG.to_string()]));

                    // Blacklist Exclude Reads Descriptor
                    let _blacklist_exclude_reads_descriptor =
                        try!(create_descriptor_with_value(&blacklist_exclude_reads_characteristic,
                                                     BLACKLIST_EXCLUDE_READS_DESCRIPTOR_UUID.to_owned(),
                                                     vec![054, 054, 054]));

                    // Blacklist Descriptor
                    let _blacklist_descriptor =
                        try!(create_descriptor_with_value(&blacklist_exclude_reads_characteristic,
                                                     BLACKLIST_DESCRIPTOR_UUID.to_owned(),
                                                     vec![054, 054, 054]));

                    // Device Information Service
                    let device_information_service =
                        try!(create_service(&connectable_device,
                                       DEVICE_INFORMATION_UUID.to_owned()));

                    // Serial Number String Characteristic
                    let _serial_number_string_characteristic =
                        try!(create_characteristic(&device_information_service,
                                              SERIAL_NUMBER_STRING_UUID.to_owned()));

                    // Generic Access Service
                    let _generic_access_service = try!(create_generic_access_service(&connectable_device));

                    // Heart Rate Service
                    let _heart_rate_service = try!(create_heart_rate_service(&connectable_device));

                    // Human Interface Device Service
                    let _human_interface_device_service =
                        try!(create_service(&connectable_device,
                                       HUMAN_INTERFACE_DEVICE_SERVICE_UUID.to_owned()));
                },
                _ => return Err(Box::from(WRONG_DATA_SET_ERROR.to_string())),
            }
        },
        None => return Err(Box::from(ADAPTER_ERROR.to_string())),
    }
    return Ok(());
}

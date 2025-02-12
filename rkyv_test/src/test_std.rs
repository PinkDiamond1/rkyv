#[cfg(test)]
mod tests {
    use crate::util::alloc::*;
    use rkyv::{AlignedBytes, Archive, Deserialize, Serialize, archived_root, ser::{serializers::WriteSerializer, Serializer}};
    use std::collections::HashMap;

    #[cfg(feature = "wasm")]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn write_serializer() {
        #[derive(Archive, Serialize)]
        #[archive_attr(repr(C))]
        struct Example {
            x: i32,
        }

        let mut buf = AlignedBytes([0u8; 3]);
        let mut ser = WriteSerializer::new(&mut buf[..]);
        let foo = Example { x: 100 };
        ser.serialize_value(&foo)
            .expect_err("serialized to an undersized buffer must fail");
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn archive_hash_map() {
        #[cfg(not(any(feature = "archive_le", feature = "archive_be")))]
        {
            test_archive(&HashMap::<i32, i32>::new());

            let mut hash_map = HashMap::new();
            hash_map.insert(1, 2);
            hash_map.insert(3, 4);
            hash_map.insert(5, 6);
            hash_map.insert(7, 8);

            test_archive(&hash_map);
        }

        let mut hash_map = HashMap::new();
        hash_map.insert("hello".to_string(), "world".to_string());
        hash_map.insert("foo".to_string(), "bar".to_string());
        hash_map.insert("baz".to_string(), "bat".to_string());

        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&hash_map).unwrap();
        let buf = serializer.into_serializer().into_inner();
        let archived_value = unsafe { archived_root::<HashMap<String, String>>(buf.as_ref()) };

        assert_eq!(archived_value.len(), hash_map.len());

        for (key, value) in hash_map.iter() {
            assert!(archived_value.contains_key(key.as_str()));
            assert_eq!(&archived_value[key.as_str()], value);
        }

        for (key, value) in archived_value.iter() {
            assert!(hash_map.contains_key(key.as_str()));
            assert_eq!(&hash_map[key.as_str()], value);
        }
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    #[allow(deprecated)]
    fn archive_hash_map_hasher() {
        use std::collections::HashMap;

        test_archive(&HashMap::<i8, i32, ahash::RandomState>::default());

        let mut hash_map: HashMap<i8, _, ahash::RandomState> = HashMap::default();
        hash_map.insert(1, 2);
        hash_map.insert(3, 4);
        hash_map.insert(5, 6);
        hash_map.insert(7, 8);

        test_archive(&hash_map);

        let mut hash_map: HashMap<_, _, ahash::RandomState> = HashMap::default();
        hash_map.insert("hello".to_string(), "world".to_string());
        hash_map.insert("foo".to_string(), "bar".to_string());
        hash_map.insert("baz".to_string(), "bat".to_string());

        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&hash_map).unwrap();
        let buf = serializer.into_serializer().into_inner();
        let archived_value =
            unsafe { archived_root::<HashMap<String, String, ahash::RandomState>>(buf.as_ref()) };

        assert_eq!(archived_value.len(), hash_map.len());

        for (key, value) in hash_map.iter() {
            assert!(archived_value.contains_key(key.as_str()));
            assert_eq!(&archived_value[key.as_str()], value);
        }

        for (key, value) in archived_value.iter() {
            assert!(hash_map.contains_key(key.as_str()));
            assert_eq!(&hash_map[key.as_str()], value);
        }
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn archive_net() {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[archive(compare(PartialEq))]
        #[archive_attr(derive(Debug))]
        struct TestNet {
            ipv4: Ipv4Addr,
            ipv6: Ipv6Addr,
            ip: IpAddr,
            sockv4: SocketAddrV4,
            sockv6: SocketAddrV6,
            sock: SocketAddr,
        }

        let value = TestNet {
            ipv4: Ipv4Addr::new(31, 41, 59, 26),
            ipv6: Ipv6Addr::new(31, 41, 59, 26, 53, 58, 97, 93),
            ip: IpAddr::V4(Ipv4Addr::new(31, 41, 59, 26)),
            sockv4: SocketAddrV4::new(Ipv4Addr::new(31, 41, 59, 26), 5358),
            sockv6: SocketAddrV6::new(Ipv6Addr::new(31, 31, 59, 26, 53, 58, 97, 93), 2384, 0, 0),
            sock: SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::new(31, 31, 59, 26, 53, 58, 97, 93),
                2384,
                0,
                0,
            )),
        };

        test_archive(&value);
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn c_string() {
        use std::ffi::CString;

        let value = unsafe { CString::from_vec_unchecked("hello world".to_string().into_bytes()) };
        test_archive(&value);
    }

    // TODO: figure out errors

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn mutex() {
    //     use rkyv::with::Lock;
    //     use std::sync::Mutex;

    //     #[derive(Archive, Serialize, Deserialize)]
    //     struct Test {
    //         #[with(Lock)]
    //         value: Mutex<i32>,
    //     }

    //     let value = Test {
    //         value: Mutex::new(10),
    //     };
    //     let mut serializer = AlignedSerializer::new(AlignedVec::new());
    //     serializer.serialize_value(&value).unwrap();
    //     let result = serializer.into_inner();
    //     let archived = unsafe { archived_root::<Test>(result.as_slice()) };

    //     assert_eq!(*archived.value, 10);

    //     let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();

    //     assert_eq!(*deserialized.value.lock().unwrap(), 10);
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn rwlock() {
    //     use rkyv::with::Lock;
    //     use std::sync::RwLock;

    //     #[derive(Archive, Serialize, Deserialize)]
    //     struct Test {
    //         #[with(Lock)]
    //         value: RwLock<i32>,
    //     }

    //     let value = Test {
    //         value: RwLock::new(10),
    //     };
    //     let mut serializer = AlignedSerializer::new(AlignedVec::new());
    //     serializer.serialize_value(&value).unwrap();
    //     let result = serializer.into_inner();
    //     let archived = unsafe { archived_root::<Test>(result.as_slice()) };

    //     assert_eq!(*archived.value, 10);

    //     let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();

    //     assert_eq!(*deserialized.value.read().unwrap(), 10);
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn os_string() {
    //     use rkyv::with::ToString;
    //     use core::str::FromStr;
    //     use std::ffi::OsString;

    //     #[derive(Archive, Serialize, Deserialize)]
    //     struct Test {
    //         #[with(ToString)]
    //         value: OsString,
    //     }

    //     let value = Test {
    //         value: OsString::from_str("hello world").unwrap(),
    //     };
    //     let mut serializer = AlignedSerializer::new(AlignedVec::new());
    //     serializer.serialize_value(&value).unwrap();
    //     let result = serializer.into_inner();
    //     let archived = unsafe { archived_root::<Test>(result.as_slice()) };

    //     assert_eq!(archived.value, "hello world");

    //     let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();

    //     assert_eq!(deserialized.value, "hello world");
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn path_buf() {
    //     use rkyv::with::ToString;
    //     use core::str::FromStr;
    //     use std::path::PathBuf;

    //     #[derive(Archive, Serialize, Deserialize)]
    //     struct Test {
    //         #[with(ToString)]
    //         value: PathBuf,
    //     }

    //     let value = Test {
    //         value: PathBuf::from_str("hello world").unwrap(),
    //     };
    //     let mut serializer = AlignedSerializer::new(AlignedVec::new());
    //     serializer.serialize_value(&value).unwrap();
    //     let result = serializer.into_inner();
    //     let archived = unsafe { archived_root::<Test>(result.as_slice()) };

    //     assert_eq!(archived.value, "hello world");

    //     let deserialized: Test = archived.deserialize(&mut Infallible).unwrap();

    //     assert_eq!(deserialized.value.to_str().unwrap(), "hello world");
    // }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // Don't run these tests with non-native endianness because ArchivedHashMap won't have
    // PartialEq<HashMap>
    #[cfg(not(any(feature = "archive_le", feature = "archive_be")))]
    fn archive_zst_containers() {
        use std::collections::HashSet;

        #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
        #[archive(compare(PartialEq))]
        #[archive_attr(derive(Debug))]
        struct MyZST;

        let mut value = HashMap::new();
        value.insert(0, ());
        value.insert(1, ());
        test_archive(&value);

        let mut value = HashMap::new();
        value.insert((), 10);
        test_archive(&value);

        let mut value = HashMap::new();
        value.insert((), ());
        test_archive(&value);

        let mut value = HashSet::new();
        value.insert(());
        test_archive(&value);
    }
}

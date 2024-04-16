use std::net::SocketAddr;

#[cfg(target_os = "windows")]
use std::os::windows::io::{AsRawSocket, AsSocket};

#[cfg(target_os = "linux")]
use std::ffi::CStr;

#[cfg(not(target_os = "windows"))]
use net_route::Route;

use socket2::{Domain, SockAddr, Type};

#[tokio::main]
async fn main() {
    let tun_name = "utun6";
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
    ctrlc2::set_async_handler(async move {
        tx.send(())
            .await
            .expect("Could not send signal on channel.");
    })
    .await;

    let mut tun_config = tun2::Configuration::default();
    tun_config
        .address((10, 0, 0, 1))
        .destination((10, 0, 0, 9)) //Windows下不能是10.0.0.255否则无法路由全局流量
        .netmask((255, 255, 255, 0))
        .tun_name(tun_name)
        .mtu(1500)
        .up();
    let mut stack_config = ipstack::IpStackConfig::default();
    stack_config.mtu(1500);
    let mut ipstack =
        ipstack::IpStack::new(stack_config, tun2::create_as_async(&tun_config).unwrap());

    // let args = tproxy_config::TproxyArgs::new()
    //     .tun_name(tun_name)
    //     .tun_mtu(1500)
    //     .tun_ip(IpAddr::from(([10, 0, 0, 1])))
    //     .tun_netmask(IpAddr::from(([255, 255, 255, 0])))
	// 	.tun_gateway(IpAddr::from([10, 0, 0, 255]));
    // let route_state = tproxy_config::tproxy_setup(&args).unwrap();

    #[cfg(target_family = "unix")]
    let tun_index = {
        use std::ffi::CString;
        let name = CString::new(tun_name).unwrap();
        unsafe { libc::if_nametoindex(name.as_ptr()) }
    };

    let handle = net_route::Handle::new().unwrap();

	let outbound_index = {
		let default = handle.default_route().await.unwrap().unwrap();
		default.ifindex.unwrap()
	};

	#[cfg(target_os = "linux")]
	let outbound_name = {
		let mut name = [0i8;128];
		unsafe {
			let ptr = name.as_mut_ptr();
			let s = libc::if_indextoname(outbound_index, ptr);
			let name = CStr::from_ptr(s).to_owned().to_str().unwrap().to_owned();
			name
		}
	};

	//handle.delete(&Route::new(IpAddr::from([10, 0, 0, 0]), 24).with_ifindex(tun_index).with_gateway(IpAddr::from([10,0,0,255]))).await;
	#[cfg(target_os = "linux")]
	let routes = [
		Route::new(IpAddr::from([0, 0, 0, 0]), 1).with_ifindex(tun_index),  // does not work on macOS
        Route::new(IpAddr::from([128, 0, 0, 0]), 1).with_ifindex(tun_index),
    ];

	#[cfg(target_os = "macos")]
    let routes = [
		// Route::new(IpAddr::from([0, 0, 0, 0]), 1).with_ifindex(tun_index),  // does not work on macOS
        Route::new(IpAddr::from([1, 0, 0, 0]), 8).with_ifindex(tun_index),
		Route::new(IpAddr::from([2, 0, 0, 0]), 7).with_ifindex(tun_index),
		Route::new(IpAddr::from([4, 0, 0, 0]), 6).with_ifindex(tun_index),
		Route::new(IpAddr::from([8, 0, 0, 0]), 5).with_ifindex(tun_index),
		Route::new(IpAddr::from([16, 0, 0, 0]), 4).with_ifindex(tun_index),
		Route::new(IpAddr::from([32, 0, 0, 0]), 3).with_ifindex(tun_index),
		Route::new(IpAddr::from([64, 0, 0, 0]), 2).with_ifindex(tun_index),
        Route::new(IpAddr::from([128, 0, 0, 0]), 1).with_ifindex(tun_index),
    ];

    #[cfg(target_os = "windows")]
    let routes = [
    ];

    for r in &routes {
        handle.add(r).await.unwrap();
    }

    tokio::spawn(async move {
        while let Ok(stream) = ipstack.accept().await {
            match stream {
                ipstack::stream::IpStackStream::Tcp(mut tcp) => {
                    println!("src {} -> dest {}", tcp.local_addr(), tcp.peer_addr());
                    // if tcp.peer_addr().ip().to_string() == "101.35.230.139" {
                    //     panic!("loop routing");
                    // }
                    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
                    #[cfg(target_os = "linux")]
                    socket.bind_device(Some(outbound_name.as_bytes())).unwrap();

                    #[cfg(target_os = "macos")]
                    {
                        use std::num::NonZeroU32;
                        // let index = {
                        //     let out_name = CString::new("en0").unwrap();
                        //     NonZeroU32::new(unsafe { libc::if_nametoindex(out_name.as_ptr()) })
                        // };
                        // socket.bind_device_by_index_v6(index).unwrap();
                        socket.bind_device_by_index_v4(Some(NonZeroU32::new(outbound_index)).unwrap()).unwrap();
						// unsafe {
						// 	let index = index.unwrap().get();
						// 	if libc::setsockopt(socket.as_fd().as_raw_fd(), libc::IPPROTO_IP, libc::IP_BOUND_IF, std::ptr::addr_of!(index).cast(), std::mem::size_of::<u32>() as libc::socklen_t) == -1{
						// 		panic!("setsockopt error");
						// 	}
						// };
						//socket.bind(&socket2::SockAddr::from(SocketAddr::from(([192,168,1,1],0))));
                    }
                    #[cfg(target_os = "windows")]
                    {
                        use windows::Win32::Networking::WinSock;
                        //use windows::Win32::Networking::WinSock::SOCKET;
                        let sock = WinSock::SOCKET(socket.as_socket().as_raw_socket() as usize);
                        let big = outbound_index.to_be_bytes();
                        unsafe{
                            let r = WinSock::setsockopt(sock, 0, WinSock::IP_UNICAST_IF, Some(&big));
                            println!("setsockopt r = {r}");
                        }
                    }
                    socket
                        .connect(&SockAddr::from(SocketAddr::from((
                            [101, 35, 230, 139],
                            8080,
                        ))))
                        .unwrap();
                    socket.set_nonblocking(true).unwrap();
                    let mut socket = tokio::net::TcpStream::from_std(socket.into()).unwrap();
                    tokio::spawn(async move {
                        match tokio::io::copy_bidirectional(&mut tcp, &mut socket)
                            .await{
								Ok(_v) => {},
								Err(_) => {},
							}
                    });
                }
                ipstack::stream::IpStackStream::Udp(_) => {}
                ipstack::stream::IpStackStream::UnknownTransport(_) => {}
                ipstack::stream::IpStackStream::UnknownNetwork(_) => {}
            }
        }
    });
    rx.recv().await.expect("Could not receive from channel.");
    // tproxy_config::tproxy_remove(Some(route_state)).unwrap();

    for r in &routes {
        handle.delete(r).await.unwrap();
    }

    println!("Got it! Exiting...");
}

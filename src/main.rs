use std::{
    ffi::CString,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
};

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
        .destination((10, 0, 0, 255))
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

    let tun_index = {
        let name = CString::new(tun_name).unwrap();
        unsafe { libc::if_nametoindex(name.as_ptr()) }
    };

    let handle = net_route::Handle::new().unwrap();
	handle.delete(&Route::new(IpAddr::from([10, 0, 0, 0]), 24).with_ifindex(tun_index).with_gateway(IpAddr::from([10,0,0,255]))).await;
    let routes = [
        Route::new(IpAddr::from([0, 0, 0, 0]), 1).with_gateway(IpAddr::from([10,0,0,255])),
        Route::new(IpAddr::from([128, 0, 0, 0]), 1).with_ifindex(tun_index),
    ];
    for r in &routes {
        handle.add(r).await.unwrap();
    }

    tokio::spawn(async move {
        while let Ok(stream) = ipstack.accept().await {
            match stream {
                ipstack::stream::IpStackStream::Tcp(mut tcp) => {
                    println!("src {} -> dest {}", tcp.local_addr(), tcp.peer_addr());
                    if tcp.peer_addr().ip().to_string() == "101.35.230.139" {
                        panic!("loop routing");
                    }
                    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
                    #[cfg(target_os = "linux")]
                    socket.bind_device(Some(b"enp4s0")).unwrap();
                    #[cfg(target_os = "macos")]
                    {
                        let index = {
                            let out_name = CString::new("en0").unwrap();
                            NonZeroU32::new(unsafe { libc::if_nametoindex(out_name.as_ptr()) })
                        };
                        //socket.bind_device_by_index_v6(index).unwrap();
                        socket.bind_device_by_index_v4(index).unwrap();
						//socket.bind(&socket2::SockAddr::from(SocketAddr::from(([192,168,2,1],8080))));
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
                        tokio::io::copy_bidirectional(&mut tcp, &mut socket)
                            .await
                            .unwrap();
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

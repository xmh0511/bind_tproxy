use std::{
    ffi::CString,
    net::{IpAddr, SocketAddr},
};

use net_route::Route;
use socket2::{Domain, SockAddr, Type};

#[tokio::main]
async fn main() {
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
        .tun_name("utun1")
        .mtu(1500)
        .up();
    let mut stack_config = ipstack::IpStackConfig::default();
    stack_config.mtu(1500);
    let mut ipstack =
        ipstack::IpStack::new(stack_config, tun2::create_as_async(&tun_config).unwrap());

    // let args = tproxy_config::TproxyArgs::new().tun_name("utun1").tun_mtu(1500);
    // let route_state= tproxy_config::tproxy_setup(&args).unwrap();

    let tun_index = {
        let name = CString::new("utun1").unwrap();
        unsafe { libc::if_nametoindex(name.as_ptr()) }
    };

    let handle = net_route::Handle::new().unwrap();
    let routes = [
        Route::new(IpAddr::from([0, 0, 0, 0]), 1).with_ifindex(tun_index),
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
                    socket.bind_device(Some(b"enp4s0")).unwrap();
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

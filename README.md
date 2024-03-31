```
Internet:
Destination        Gateway            Flags           Netif Expire
default            10.0.0.255         UGScg           utun6       
default            192.168.2.1        UGScIg            en0       
10/24              10.0.0.255         UGSc            utun6       
10.0.0.255         10.0.0.1           UHr             utun6       
10.147.17/24       link#7             UC           feth1532      !
127                127.0.0.1          UCS               lo0       
127.0.0.1          127.0.0.1          UH                lo0       
169.254            link#5             UCS               en0      !
192.168.2          link#5             UCS               en0      !
192.168.2.1/32     link#5             UCS               en0      !
192.168.2.1        2c:a0:42:b9:39:72  UHLWIir           en0   1160
192.168.2.21/32    link#5             UCS               en0      !
224.0.0/4          link#5             UmCS              en0      !
224.0.0.251        1:0:5e:0:0:fb      UHmLWI            en0       
239.255.255.250    1:0:5e:7f:ff:fa    UHmLWI            en0       
255.255.255.255/32 link#5             UCS               en0      !




-------------------------------

Internet:
Destination        Gateway            Flags           Netif Expire
0/1                link#17            UScg            utun6       
default            192.168.2.1        UGScg             en0       
10/24              10.0.0.255         UGSc            utun6       
10.0.0.255         10.0.0.1           UH              utun6       
10.147.17/24       link#7             UC           feth1532      !
127                127.0.0.1          UCS               lo0       
127.0.0.1          127.0.0.1          UH                lo0       
128.0/1            link#17            USc             utun6       
169.254            link#5             UCS               en0      !
192.168.2          link#5             UCS               en0      !
192.168.2.1/32     link#5             UCS               en0      !
192.168.2.1        2c:a0:42:b9:39:72  UHLWIir           en0   1168
192.168.2.21/32    link#5             UCS               en0      !
224.0.0/4          link#5             UmCS              en0      !
224.0.0.251        1:0:5e:0:0:fb      UHmLWI            en0       
239.255.255.250    1:0:5e:7f:ff:fa    UHmLWI            en0       
255.255.255.255/32 link#5             UCS               en0      !
```

`bind_device_by_index_v4` can work on the first but not the second table setting. 

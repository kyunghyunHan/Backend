use std::io;
use std::net::UdpSocket;
/*
수신자
239.0.0.1포트에 가입
0.0.0.0 : 모든 네트워크 인퍼페이스로받기
192.168.1.100 : 이 네트워크 인터페이스로 만 받기

[컴퓨터1]
├── 유선 랜카드: 192.168.1.100
├── 무선 랜카드: 192.168.2.100
└── 다른 네트워크 카드: 10.0.0.100

네트워크 인터페이스 에 해당하는 IP가 있어야함
*/
fn receiver() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8888")?;
    //239.0.0.1의 메세지를  모든 네트워크 인터페이스 사용
    //멀티캐스트 그룹 + 사용할 네트워크 인터페이스
    socket.join_multicast_v4(&"239.0.0.1".parse().unwrap(), &"0.0.0.0".parse().unwrap())?;
    let mut buffer = [0u8; 1024]; //데이터 저장 공간
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                //utf-8 문자열로 변환
                //&buffer[..size] : buffer slice
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("받은 메세지 {}: {}", addr, message);
            }
            Err(e) => eprintln!("에러: {}", e),
        }
    }
}

fn sender() -> io::Result<()> {
    //:0 -> 운영체제가 사용가능한 포트를 자동으로 할당
    //송신주소
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    //TTL설정
    socket.set_multicast_ttl_v4(5)?; //최대 5개의 라우터 까지만 전달

    // socket.set_multicast_ttl_v4(1)?;  // 같은 서브넷으로만 제한

    // // 조직 내부로 제한
    // socket.set_multicast_ttl_v4(32)?; // 같은 사이트로 제한

    // // 글로벌 전송 허용
    // socket.set_multicast_ttl_v4(255)?; // 제한 없음
    //멀티캐스트 그룹주소
    let muticast_addr = "239.0.0.1:8888";
    let message = "Hello Multicast Would";

    loop {
        //멀티캐스트 주소로 송신
        match socket.send_to(message.as_bytes(), muticast_addr) {
            Ok(_) => {
                println!("멀티캐스트 메세지");
            }

            Err(e) => {
                eprintln!("Faild to send:{}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1))
    }
}

pub fn example() {
    sender().unwrap();
}

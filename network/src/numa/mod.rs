use std::ffi::CString;

pub fn example() {
    println!("🔧 libc 간단 예제들");
    
    // 1. 프로세스 ID 가져오기
    process_info();
    
    // 2. 시스템 정보
    system_info();
    
    // 3. CPU 친화성 설정
    cpu_affinity_example();
    
    // 4. 메모리 잠금 (HFT에서 중요!)
    memory_lock_example();
    
    // 5. 파일 시스템 작업
    file_system_example();
}

// 1. 프로세스 정보
fn process_info() {
    println!("=== 프로세스 정보 ===");
    
    unsafe {
        let pid = libc::getpid();
        let ppid = libc::getppid();
        let uid = libc::getuid();
        
        println!("프로세스 ID: {}", pid);
        println!("부모 프로세스 ID: {}", ppid);
        println!("사용자 ID: {}", uid);
    }
    println!();
}

// 2. 시스템 정보
fn system_info() {
    println!("=== 시스템 정보 ===");
    
    unsafe {
        // CPU 개수
        let cpu_count = libc::sysconf(libc::_SC_NPROCESSORS_ONLN);
        println!("CPU 코어 수: {}", cpu_count);
        
        // 페이지 크기
        let page_size = libc::sysconf(libc::_SC_PAGESIZE);
        println!("메모리 페이지 크기: {} bytes", page_size);
        
        // 물리 메모리 크기
        let phys_pages = libc::sysconf(libc::_SC_PHYS_PAGES);
        let total_memory = phys_pages * page_size;
        println!("총 물리 메모리: {} bytes ({:.2} GB)", 
            total_memory, total_memory as f64 / 1024.0 / 1024.0 / 1024.0);
    }
    println!();
}

// 3. CPU 친화성 설정 (플랫폼별 분기)
fn cpu_affinity_example() {
    println!("=== CPU 친화성 설정 ===");
    
    #[cfg(target_os = "linux")]
    {
        linux_cpu_affinity();
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        println!("❌ CPU 친화성은 Linux에서만 지원됩니다");
        println!("현재 운영체제: {}", std::env::consts::OS);
    }
    
    println!();
}

#[cfg(target_os = "linux")]
fn linux_cpu_affinity() {
    use std::mem;
    
    unsafe {
        // Linux에서만 사용 가능
        let mut cpu_set: libc::cpu_set_t = mem::zeroed();
        
        // CPU_SET 매크로 대신 직접 비트 조작
        let cpu_setsize = mem::size_of::<libc::cpu_set_t>();
        
        // CPU 0, 1, 2번 설정을 위한 비트마스크
        // Linux에서 cpu_set_t는 보통 비트마스크 배열
        let cpu_mask = 0b111; // CPU 0, 1, 2 (첫 3비트)
        std::ptr::write(&mut cpu_set as *mut _ as *mut u64, cpu_mask);
        
        let result = libc::sched_setaffinity(
            0, // 현재 스레드
            cpu_setsize,
            &cpu_set
        );
        
        if result == 0 {
            println!("✅ CPU 0, 1, 2번에 스레드 바인딩 성공");
        } else {
            let error = *libc::__errno_location();
            println!("❌ CPU 바인딩 실패, 에러코드: {}", error);
        }
    }
}

// 4. 메모리 잠금 (HFT에서 스왑 방지)
fn memory_lock_example() {
    println!("=== 메모리 잠금 (HFT 최적화) ===");
    
    // 중요한 데이터 (주문, 시세 등)
    let important_data = vec![1, 2, 3, 4, 5];
    
    unsafe {
        // 특정 메모리 페이지를 RAM에 고정 (스왑 방지)
        let ptr = important_data.as_ptr() as *mut libc::c_void;
        let len = important_data.len() * std::mem::size_of::<i32>();
        
        let result = libc::mlock(ptr, len);
        
        if result == 0 {
            println!("✅ 메모리 잠금 성공 - 스왑되지 않음");
            println!("데이터 크기: {} bytes", len);
            
            // 메모리 잠금 해제
            libc::munlock(ptr, len);
            println!("🔓 메모리 잠금 해제");
        } else {
            println!("❌ 메모리 잠금 실패 (권한 필요: sudo)");
        }
        
        // 전체 프로세스 메모리 잠금 (더 강력함)
        println!("전체 프로세스 메모리 잠금 시도...");
        let result = libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE);
        
        if result == 0 {
            println!("✅ 전체 메모리 잠금 성공!");
            libc::munlockall();
            println!("🔓 전체 메모리 잠금 해제");
        } else {
            println!("❌ 전체 메모리 잠금 실패 (sudo 권한 필요)");
        }
    }
    println!();
}

// 5. 파일 시스템 저수준 작업
fn file_system_example() {
    println!("=== 파일 시스템 저수준 작업 ===");
    
    let filename = CString::new("test_file.txt").unwrap();
    
    unsafe {
        // 파일 생성/열기
        let fd = libc::open(
            filename.as_ptr(),
            libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
            0o644
        );
        
        if fd != -1 {
            println!("✅ 파일 생성 성공, File Descriptor: {}", fd);
            
            // 데이터 쓰기
            let data = b"Hello from libc!\n";
            let bytes_written = libc::write(
                fd,
                data.as_ptr() as *const libc::c_void,
                data.len()
            );
            
            println!("📝 {} bytes 쓰기 완료", bytes_written);
            
            // 파일 동기화 (즉시 디스크에 저장)
            libc::fsync(fd);
            println!("💾 파일 동기화 완료");
            
            // 파일 닫기
            libc::close(fd);
            println!("🔒 파일 닫기 완료");
            
            // 파일 정보 확인
            let mut stat: libc::stat = std::mem::zeroed();
            let result = libc::stat(filename.as_ptr(), &mut stat);
            
            if result == 0 {
                println!("📊 파일 정보:");
                println!("  - 크기: {} bytes", stat.st_size);
                println!("  - 권한: {:o}", stat.st_mode & 0o777);
                println!("  - 수정 시간: {}", stat.st_mtime);
            }
            
            // 파일 삭제
            libc::unlink(filename.as_ptr());
            println!("🗑️  파일 삭제 완료");
            
        } else {
            println!("❌ 파일 생성 실패");
        }
    }
    println!();
}

/*
HFT에서 libc가 중요한 이유:

1. 성능 최적화:
   - CPU 친화성 설정으로 컨텍스트 스위칭 최소화
   - 메모리 잠금으로 스왑 방지 (지연시간 안정화)
   - 직접적인 시스템 콜로 오버헤드 제거

2. 실시간 요구사항:
   - 스케줄러 우선순위 설정 (SCHED_FIFO, SCHED_RR)
   - 실시간 신호 처리
   - 정밀한 타이밍 제어

3. 네트워크 최적화:
   - 소켓 옵션 세밀 조정
   - 네트워크 인터럽트 제어
   - TCP_NODELAY, SO_REUSEPORT 등

실행할 때:
sudo ./program  # 메모리 잠금 기능을 위해 root 권한 필요
*/
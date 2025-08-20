//SIMD 
//Single Instruction, Multiple Data
use std::time::Instant;

fn add_numbers_slow(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    
    // 하나씩 순서대로 처리
    for i in 0..a.len() {
        result.push(a[i] + b[i]); // 1번에 1개씩만 계산
    }
    
    result
}

// Apple Silicon (M1/M2/M3) NEON SIMD 사용
#[cfg(target_arch = "aarch64")]
fn add_numbers_fast_neon(a: &[i32], b: &[i32]) -> Vec<i32> {
    use std::arch::aarch64::*;
    
    let mut result = Vec::with_capacity(a.len());
    let chunks = a.len() / 4; // NEON은 128비트 = 4개의 i32를 한 번에
    
    unsafe {
        // 4개씩 묶어서 처리
        for i in 0..chunks {
            let offset = i * 4;
            
            // 4개의 i32를 한 번에 메모리에서 로드
            let va = vld1q_s32(a.as_ptr().add(offset));
            let vb = vld1q_s32(b.as_ptr().add(offset));
            
            // 4개를 동시에 덧셈! 🚀
            let vresult = vaddq_s32(va, vb);
            
            // 결과를 저장
            let mut temp = [0i32; 4];
            vst1q_s32(temp.as_mut_ptr(), vresult);
            result.extend_from_slice(&temp);
        }
        
        // 나머지 개별 처리
        for i in chunks * 4..a.len() {
            result.push(a[i] + b[i]);
        }
    }
    
    result
}

pub fn simd_i32_demo() {
    println!("🧮 SIMD i32 벡터 덧셈 데모");
    println!("========================");
    
    // 테스트 데이터 생성
    let size = 1_000_000; // 백만 개
    let a: Vec<i32> = (0..size).collect();
    let b: Vec<i32> = (1000..size + 1000).collect();
    
    println!("📊 벡터 크기: {} 개의 i32", size);
    println!();
    
    // 일반 처리 (느림)
    println!("🐌 일반 처리 (하나씩):");
    let start = Instant::now();
    let result_slow = add_numbers_slow(&a, &b);
    let slow_time = start.elapsed();
    println!("   시간: {:?}", slow_time);
    println!("   처리 방식: 1번에 1개씩 덧셈");
    println!();
    
    // SIMD 처리 (빠름)
    #[cfg(target_arch = "aarch64")]
    {
        println!("🚀 Apple Silicon NEON SIMD:");
        let start = Instant::now();
        let result_fast = add_numbers_fast_neon(&a, &b);
        let fast_time = start.elapsed();
        
        println!("   시간: {:?}", fast_time);
        println!("   처리 방식: 1번에 4개씩 동시 덧셈");
        println!("   🎯 속도 향상: {:.1}배 빨라짐!", 
                slow_time.as_nanos() as f64 / fast_time.as_nanos() as f64);
        
        // 결과가 정확한지 확인
        let is_correct = result_slow == result_fast;
        println!("   ✅ 결과 정확성: {}", if is_correct { "정확함" } else { "오차 있음" });
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        println!("🚀 Intel SIMD:");
        let start = Instant::now();
        let result_fast = add_numbers_fast_intel(&a, &b);
        let fast_time = start.elapsed();
        
        println!("   시간: {:?}", fast_time);
        
        if is_x86_feature_detected!("avx2") {
            println!("   처리 방식: AVX2로 1번에 8개씩 동시 덧셈");
        } else if is_x86_feature_detected!("sse2") {
            println!("   처리 방식: SSE2로 1번에 4개씩 동시 덧셈");
        }
        
        println!("   🎯 속도 향상: {:.1}배 빨라짐!", 
                slow_time.as_nanos() as f64 / fast_time.as_nanos() as f64);
        
        // 결과가 정확한지 확인
        let is_correct = result_slow == result_fast;
        println!("   ✅ 결과 정확성: {}", if is_correct { "정확함" } else { "오차 있음" });
    }
    
    println!();
    println!("💡 SIMD의 원리:");
    println!("   • 일반: [1+1001] [2+1002] [3+1003] [4+1004] <- 4번의 연산");
    println!("   • SIMD: [1,2,3,4] + [1001,1002,1003,1004] <- 1번의 연산으로 4개 처리!");
    println!("   • CPU가 한 번의 명령으로 여러 i32를 동시 처리");
    
    // 간단한 예시
    println!();
    println!("🔍 간단한 예시:");
    let small_a = vec![1, 2, 3, 4, 5];
    let small_b = vec![10, 20, 30, 40, 50];
    let small_result = add_numbers_slow(&small_a, &small_b);
    
    println!("   a = {:?}", small_a);
    println!("   b = {:?}", small_b);
    println!("   결과 = {:?}", small_result);
}
fn check_my_cpu_features() {
    println!("🍎 M1 Mac (Apple Silicon) CPU의 SIMD 지원 현황:");
    
    // Apple Silicon (ARM64/aarch64) 확인
    #[cfg(target_arch = "aarch64")]
    {
        println!("✅ NEON 지원 (128비트) - M1/M2/M3 기본 탑재!");
        println!("✅ Advanced SIMD 지원");
        println!("✅ AES 암호화 가속 지원");
        println!("✅ SHA 해시 가속 지원");
        
        // 컴파일 시 NEON이 활성화되었는지 확인
        #[cfg(target_feature = "neon")]
        println!("✅ NEON이 컴파일 시 활성화됨");
        
        #[cfg(not(target_feature = "neon"))]
        println!("ℹ️  NEON이 컴파일 시 비활성화됨 (런타임에는 사용 가능)");
        
        // Apple Silicon 특별 기능들
        println!("🚀 Apple Silicon 특별 기능들:");
        println!("   🧠 Neural Engine - AI/ML 하드웨어 가속");
        println!("   📊 AMX (Apple Matrix Extensions) - 행렬 연산 특화");
        println!("   🔗 통합 메모리 아키텍처 - CPU/GPU 메모리 공유");
        println!("   🔋 고효율 설계 - Intel 대비 전력 효율 우수");
        
        // M1/M2/M3 구분 (대략적)
        println!("\nℹ️  M1/M2/M3 모든 칩이 강력한 SIMD 성능을 제공합니다!");
        println!("   • M1: 8코어 CPU + 고성능 NEON");
        println!("   • M2: 개선된 효율성 + 더 빠른 NEON");  
        println!("   • M3: 3nm 공정 + 최적화된 SIMD");
    }
    
    // Intel x86_64 (Intel Mac) 확인
    #[cfg(target_arch = "x86_64")]
    unsafe {
        println!("💻 Intel Mac CPU의 SIMD 지원 현황:");
        
        if std::arch::is_x86_feature_detected!("sse") {
            println!("✅ SSE 지원 (128비트)");
        } else {
            println!("❌ SSE 미지원");
        }
        
        if std::arch::is_x86_feature_detected!("sse2") {
            println!("✅ SSE2 지원 (128비트)");
        } else {
            println!("❌ SSE2 미지원");
        }
        
        if std::arch::is_x86_feature_detected!("avx") {
            println!("✅ AVX 지원 (256비트)");
        } else {
            println!("❌ AVX 미지원");
        }
        
        if std::arch::is_x86_feature_detected!("avx2") {
            println!("✅ AVX2 지원 (256비트 개선)");
        } else {
            println!("❌ AVX2 미지원");
        }
        
        if std::arch::is_x86_feature_detected!("fma") {
            println!("✅ FMA 지원 (Fused Multiply-Add)");
        } else {
            println!("❌ FMA 미지원");
        }
        
        if std::arch::is_x86_feature_detected!("avx512f") {
            println!("✅ AVX-512 지원 (512비트) - Intel Mac에서는 드물어요!");
        } else {
            println!("❌ AVX-512 미지원 (Intel Mac에서 일반적)");
        }
        
        println!("\nℹ️  Intel Mac도 강력하지만 Apple Silicon이 더 효율적입니다!");
    }
    
    // 기타 아키텍처
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    println!("❓ 알 수 없는 아키텍처입니다.");
    
    // 시스템 정보 추가
    println!("\n📱 시스템 정보:");
    println!("   OS: {}", std::env::consts::OS);
    println!("   아키텍처: {}", std::env::consts::ARCH);
    println!("   패밀리: {}", std::env::consts::FAMILY);
}
pub fn example(){
    check_my_cpu_features();
    simd_i32_demo();
}
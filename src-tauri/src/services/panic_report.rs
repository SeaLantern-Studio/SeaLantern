use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[repr(C, align(64))]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
struct Regs {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    rsp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
    rflags: u64,

    cs: u64,
    ds: u64,
    es: u64,
    fs: u64,
    gs: u64,
    ss: u64,

    fxsave_area: [u8; 512],
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
extern "C" {
    fn getregs(regs: *mut Regs);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
static START_TIME: OnceLock<SystemTime> = OnceLock::new();

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
static PANIC_HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
#[allow(deprecated)]
pub async fn panic_report() {
    START_TIME.set(SystemTime::now()).unwrap();
    std::panic::set_hook(Box::new(|panic_info| {
        if PANIC_HOOK_RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        let mut regs = Regs {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
            cs: 0,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            ss: 0,
            fxsave_area: [0; 512],
        };
        unsafe {
            getregs(&mut regs);
        }
        let start_time = format_time(START_TIME.get().expect("start time not set").clone());
        let crash_time = format_time(SystemTime::now());
        let os_info = fs::read_to_string("/proc/version").unwrap_or_else(|_| "Unknown".to_string());
        let stack_range = get_stack_range();
        let stack_used = stack_range
            .clone()
            .map(|(_, end)| end.saturating_sub(regs.rsp))
            .unwrap_or(0);
        let cpu_temp = get_cpu_temperature();
        let mem_load = get_memory_load();
        let handle_count = fs::read_dir("/proc/self/fd")
            .map(|e| e.count())
            .unwrap_or(0);
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let script = format!(
            r#"
                panicreportname={}
                rm -r $panicreportname
                touch $panicreportname
                echo "============!Panicked!============" >> $panicreportname
                echo "===============Info===============" >> $panicreportname
                echo "Panic Time : {}" >> $panicreportname
                echo "Start Time : {}" >> $panicreportname
                echo "OS : {}" >> $panicreportname
                echo "Stack Range : {:?}" >> $panicreportname
                echo "Stack Used : {}" >> $panicreportname
                echo "CPU Temprature : {}" >> $panicreportname
                echo "Loaded Memory : {}" >> $panicreportname
                echo "Handle Counts : {}" >> $panicreportname
                echo "CPU Cores : {}" >> $panicreportname
                echo "============Registers=============" >> $panicreportname
                echo "RAX : {}" >> $panicreportname
                echo "RBX : {}" >> $panicreportname
                echo "RCX : {}" >> $panicreportname
                echo "RDX : {}" >> $panicreportname
                echo "RSI : {}" >> $panicreportname
                echo "RDI : {}" >> $panicreportname
                echo "RBP : {}" >> $panicreportname
                echo "RSP : {}" >> $panicreportname
                echo "R8  : {}" >> $panicreportname
                echo "R9  : {}" >> $panicreportname
                echo "R10 : {}" >> $panicreportname
                echo "R11 : {}" >> $panicreportname
                echo "R12 : {}" >> $panicreportname
                echo "R13 : {}" >> $panicreportname
                echo "R14 : {}" >> $panicreportname
                echo "R15 : {}" >> $panicreportname
                echo "RIP : {}" >> $panicreportname
                echo "RFLAGS : {}" >> $panicreportname
                echo "CS : {}" >> $panicreportname
                echo "DS : {}" >> $panicreportname
                echo "ES : {}" >> $panicreportname
                echo "FS : {}" >> $panicreportname
                echo "GS : {}" >> $panicreportname
                echo "SS : {}" >> $panicreportname
                echo "FXSAVEAREA : {:?}" >> $panicreportname
                echo "============Panic Info============" >> $panicreportname
                echo "{}" >> $panicreportname
                echo "============ReportEnds============" >> $panicreportname
                echo "Sea Lantern PANICKED!!"
            "#,
            shlex::quote("Panic_Report"),
            shlex::quote(&crash_time.to_string()),
            shlex::quote(&start_time.to_string()),
            shlex::quote(&os_info.to_string()),
            shlex::quote(&format!("{:?}", stack_range)),
            shlex::quote(&stack_used.to_string()),
            shlex::quote(&cpu_temp.to_string()),
            shlex::quote(&mem_load.to_string()),
            shlex::quote(&handle_count.to_string()),
            shlex::quote(&cpu_cores.to_string()),
            shlex::quote(&regs.rax.to_string()),
            shlex::quote(&regs.rbx.to_string()),
            shlex::quote(&regs.rcx.to_string()),
            shlex::quote(&regs.rdx.to_string()),
            shlex::quote(&regs.rsi.to_string()),
            shlex::quote(&regs.rdi.to_string()),
            shlex::quote(&regs.rbp.to_string()),
            shlex::quote(&regs.rsp.to_string()),
            shlex::quote(&regs.r8.to_string()),
            shlex::quote(&regs.r9.to_string()),
            shlex::quote(&regs.r10.to_string()),
            shlex::quote(&regs.r11.to_string()),
            shlex::quote(&regs.r12.to_string()),
            shlex::quote(&regs.r13.to_string()),
            shlex::quote(&regs.r14.to_string()),
            shlex::quote(&regs.r15.to_string()),
            shlex::quote(&regs.rip.to_string()),
            shlex::quote(&regs.rflags.to_string()),
            shlex::quote(&regs.cs.to_string()),
            shlex::quote(&regs.ds.to_string()),
            shlex::quote(&regs.es.to_string()),
            shlex::quote(&regs.fs.to_string()),
            shlex::quote(&regs.gs.to_string()),
            shlex::quote(&regs.ss.to_string()),
            shlex::quote(format!("{:?}", &regs.fxsave_area).as_str()),
            shlex::quote(&panic_info.to_string()),
        );

        let cmd = std::process::Command::new("bash")
            .arg("-c")
            .arg(&script)
            .status()
            .expect("");

        if cmd.success() {
            println!("PANICKED!");
        }

        std::process::exit(0xFFFF);
    }));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
fn format_time(t: SystemTime) -> String {
    let since_epoch = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let millis = since_epoch.as_millis();
    format!("{:?} ({} ms since epoch)", t, millis)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
fn get_stack_range() -> Option<(u64, u64)> {
    for line in fs::read_to_string("/proc/self/maps").ok()?.lines() {
        if line.contains("[stack]") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(range) = parts.get(0) {
                let addrs: Vec<&str> = range.split('-').collect();
                if addrs.len() == 2 {
                    let start = u64::from_str_radix(addrs[0], 16).ok()?;
                    let end = u64::from_str_radix(addrs[1], 16).ok()?;
                    return Some((start, end));
                }
            }
        }
    }
    None
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
fn get_cpu_temperature() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.to_string_lossy().contains("thermal_zone") {
                let temp_path = path.join("temp");
                if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                    if let Ok(millideg) = temp_str.trim().parse::<f64>() {
                        return format!("{:.2}", millideg / 1000.0);
                    }
                }
            }
        }
    }
    "N/A".to_string()
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(clippy::all)]
fn get_memory_load() -> f64 {
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut available = 0u64;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total = val.parse().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    available = val.parse().unwrap_or(0);
                }
            }
        }
        if total > 0 {
            return (total.saturating_sub(available) as f64 / total as f64) * 100.0;
        }
    }
    0.0
}

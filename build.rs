extern crate gcc;
extern crate serde_json;
extern crate tempdir;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use gcc::Config;
use serde_json::Value;
use tempdir::TempDir;

macro_rules! try {
    ($e:expr) => {
        $e.unwrap_or_else(|e| panic!("{} with {}", stringify!($e), e))
    }
}

/// Parsed target specification file
struct Spec(BTreeMap<String, Value>);

impl Spec {
    fn arch(&self) -> &str {
        self.mandatory("arch")
    }

    fn linker(&self) -> Option<&str> {
        self.optional("linker")
    }

    fn llvm_target(&self) -> &str {
        self.mandatory("llvm-target")
    }

    fn mandatory(&self, field: &str) -> &str {
        match self.0[field] {
            Value::String(ref s) => s,
            _ => unreachable!(),
        }
    }

    fn optional(&self, field: &str) -> Option<&str> {
        self.0.get(field).map(|field| {
            match *field {
                Value::String(ref s) => &**s,
                _ => unreachable!(),
            }
        })
    }

    fn os(&self) -> &str {
        self.mandatory("os")
    }
}

struct Target {
    name: String,
    spec: Option<Spec>,
}

impl Target {
    // TODO somehow read the specification of built-in targets. This probably requires upstream
    // (`rustc`) support.
    fn new(target: &str) -> Self {
        /// Parse `target` specification file in `dir`ectory, if it's there
        fn parse(target: &str, dir: &Path) -> Option<Spec> {
            let path = dir.join(format!("{}.json", target));

            if path.exists() {
                let json = &mut String::new();
                try!(try!(File::open(path)).read_to_string(json));

                Some(try!(serde_json::from_str(json).map(Spec)))
            } else {
                None
            }
        }

        Target {
            name: target.to_owned(),
            spec: parse(target, &try!(env::current_dir())).or_else(|| {
                env::var_os("RUST_TARGET_PATH")
                    .map(PathBuf::from)
                    .and_then(|dir| parse(target, &dir))
            }),
        }
    }

    fn arch_is(&self, arch: &str) -> bool {
        self.spec
            .as_ref()
            .map(|spec| spec.arch() == arch)
            .unwrap_or_else(|| self.name.contains(arch))
    }

    fn cpu(&self) -> Option<&str> {
        self.spec.as_ref().and_then(|spec| spec.optional("cpu"))
    }

    fn features(&self) -> Option<&str> {
        self.spec.as_ref().and_then(|spec| spec.optional("features"))
    }

    fn llvm_target(&self) -> &str {
        // TODO(unwrap_or) for *most* built-in targets, their name matches its `llvm-target` field.
        // The exceptions (e.g. aarch64-apple-ios) should be handled here.
        self.spec.as_ref().map(|spec| spec.llvm_target()).unwrap_or(&self.name)
    }

    fn os_is(&self, os: &str) -> bool {
        self.spec.as_ref().map(|spec| spec.os() == os).unwrap_or_else(|| self.name.contains(os))
    }

    fn tool(&self, env: &str, tool: &str) -> Cow<str> {
        let tool_env = &format!("{}_{}", env, self.name.replace("-", "_"));

        env::var(tool_env)
            .ok()
            .or_else(|| {
                self.spec.as_ref().and_then(|spec| spec.linker()).and_then(|linker| {
                    if linker.ends_with("gcc") {
                        Some(linker.replace("gcc", tool))
                    } else {
                        None
                    }
                })
            })
            .map(Cow::from)
            .expect(&format!("{} not set", tool_env))
    }
}

fn main() {
    let target = &Target::new(&try!(env::var("TARGET")));

    let td = try!(TempDir::new("compiler-rt"));
    let src = td.path();

    fetch(src);
    build(src, target);
}

fn fetch(td: &Path) {
    // FIXME use the `curl`, `flate2`, `tar` crates instead of shelling out to `git`.
    // FIXME Should probably use the rust-lang/compiler-rt repository
    assert!(try!(Command::new("git")
            .args(&["clone", "--depth", "1", "https://github.com/llvm-mirror/compiler-rt"])
            .arg(td)
            .status())
        .success());
}

fn build(src: &Path, target: &Target) {
    // FIXME(copied from compiler-rt source) atomic.c may only be compiled if host compiler
    // understands _Atomic
    const GENERIC_SOURCES: &'static [&'static str] = &["absvdi2.c",
                                                       "absvsi2.c",
                                                       "absvti2.c",
                                                       "adddf3.c",
                                                       "addsf3.c",
                                                       "addtf3.c",
                                                       "addvdi3.c",
                                                       "addvsi3.c",
                                                       "addvti3.c",
                                                       "apple_versioning.c",
                                                       "ashldi3.c",
                                                       "ashlti3.c",
                                                       "ashrdi3.c",
                                                       "ashrti3.c",
                                                       // "atomic.c",
                                                       "clear_cache.c",
                                                       "clzdi2.c",
                                                       "clzsi2.c",
                                                       "clzti2.c",
                                                       "cmpdi2.c",
                                                       "cmpti2.c",
                                                       "comparedf2.c",
                                                       "comparesf2.c",
                                                       "ctzdi2.c",
                                                       "ctzsi2.c",
                                                       "ctzti2.c",
                                                       "divdc3.c",
                                                       "divdf3.c",
                                                       "divdi3.c",
                                                       "divmoddi4.c",
                                                       "divmodsi4.c",
                                                       "divsc3.c",
                                                       "divsf3.c",
                                                       "divsi3.c",
                                                       "divtc3.c",
                                                       "divti3.c",
                                                       "divtf3.c",
                                                       "divxc3.c",
                                                       "enable_execute_stack.c",
                                                       "eprintf.c",
                                                       "extendsfdf2.c",
                                                       "extendhfsf2.c",
                                                       "ffsdi2.c",
                                                       "ffsti2.c",
                                                       "fixdfdi.c",
                                                       "fixdfsi.c",
                                                       "fixdfti.c",
                                                       "fixsfdi.c",
                                                       "fixsfsi.c",
                                                       "fixsfti.c",
                                                       "fixunsdfdi.c",
                                                       "fixunsdfsi.c",
                                                       "fixunsdfti.c",
                                                       "fixunssfdi.c",
                                                       "fixunssfsi.c",
                                                       "fixunssfti.c",
                                                       "fixunsxfdi.c",
                                                       "fixunsxfsi.c",
                                                       "fixunsxfti.c",
                                                       "fixxfdi.c",
                                                       "fixxfti.c",
                                                       "floatdidf.c",
                                                       "floatdisf.c",
                                                       "floatdixf.c",
                                                       "floatsidf.c",
                                                       "floatsisf.c",
                                                       "floattidf.c",
                                                       "floattisf.c",
                                                       "floattixf.c",
                                                       "floatundidf.c",
                                                       "floatundisf.c",
                                                       "floatundixf.c",
                                                       "floatunsidf.c",
                                                       "floatunsisf.c",
                                                       "floatuntidf.c",
                                                       "floatuntisf.c",
                                                       "floatuntixf.c",
                                                       "int_util.c",
                                                       "lshrdi3.c",
                                                       "lshrti3.c",
                                                       "moddi3.c",
                                                       "modsi3.c",
                                                       "modti3.c",
                                                       "muldc3.c",
                                                       "muldf3.c",
                                                       "muldi3.c",
                                                       "mulodi4.c",
                                                       "mulosi4.c",
                                                       "muloti4.c",
                                                       "mulsc3.c",
                                                       "mulsf3.c",
                                                       "multi3.c",
                                                       "multf3.c",
                                                       "mulvdi3.c",
                                                       "mulvsi3.c",
                                                       "mulvti3.c",
                                                       "mulxc3.c",
                                                       "negdf2.c",
                                                       "negdi2.c",
                                                       "negsf2.c",
                                                       "negti2.c",
                                                       "negvdi2.c",
                                                       "negvsi2.c",
                                                       "negvti2.c",
                                                       "paritydi2.c",
                                                       "paritysi2.c",
                                                       "parityti2.c",
                                                       "popcountdi2.c",
                                                       "popcountsi2.c",
                                                       "popcountti2.c",
                                                       "powidf2.c",
                                                       "powisf2.c",
                                                       "powitf2.c",
                                                       "powixf2.c",
                                                       "subdf3.c",
                                                       "subsf3.c",
                                                       "subvdi3.c",
                                                       "subvsi3.c",
                                                       "subvti3.c",
                                                       "subtf3.c",
                                                       "trampoline_setup.c",
                                                       "truncdfhf2.c",
                                                       "truncdfsf2.c",
                                                       "truncsfhf2.c",
                                                       "ucmpdi2.c",
                                                       "ucmpti2.c",
                                                       "udivdi3.c",
                                                       "udivmoddi4.c",
                                                       "udivmodsi4.c",
                                                       "udivmodti4.c",
                                                       "udivsi3.c",
                                                       "udivti3.c",
                                                       "umoddi3.c",
                                                       "umodsi3.c",
                                                       "umodti3.c"];

    const ARM_SOURCES: &'static [&'static str] = &["arm/adddf3vfp.S",
                                                   "arm/addsf3vfp.S",
                                                   "arm/aeabi_cdcmp.S",
                                                   "arm/aeabi_cdcmpeq_check_nan.c",
                                                   "arm/aeabi_cfcmp.S",
                                                   "arm/aeabi_cfcmpeq_check_nan.c",
                                                   "arm/aeabi_dcmp.S",
                                                   "arm/aeabi_div0.c",
                                                   "arm/aeabi_drsub.c",
                                                   "arm/aeabi_fcmp.S",
                                                   "arm/aeabi_frsub.c",
                                                   "arm/aeabi_idivmod.S",
                                                   "arm/aeabi_ldivmod.S",
                                                   "arm/aeabi_memcmp.S",
                                                   "arm/aeabi_memcpy.S",
                                                   "arm/aeabi_memmove.S",
                                                   "arm/aeabi_memset.S",
                                                   "arm/aeabi_uidivmod.S",
                                                   "arm/aeabi_uldivmod.S",
                                                   "arm/bswapdi2.S",
                                                   "arm/bswapsi2.S",
                                                   "arm/clzdi2.S",
                                                   "arm/clzsi2.S",
                                                   "arm/comparesf2.S",
                                                   "arm/divdf3vfp.S",
                                                   "arm/divmodsi4.S",
                                                   "arm/divsf3vfp.S",
                                                   "arm/divsi3.S",
                                                   "arm/eqdf2vfp.S",
                                                   "arm/eqsf2vfp.S",
                                                   "arm/extendsfdf2vfp.S",
                                                   "arm/fixdfsivfp.S",
                                                   "arm/fixsfsivfp.S",
                                                   "arm/fixunsdfsivfp.S",
                                                   "arm/fixunssfsivfp.S",
                                                   "arm/floatsidfvfp.S",
                                                   "arm/floatsisfvfp.S",
                                                   "arm/floatunssidfvfp.S",
                                                   "arm/floatunssisfvfp.S",
                                                   "arm/gedf2vfp.S",
                                                   "arm/gesf2vfp.S",
                                                   "arm/gtdf2vfp.S",
                                                   "arm/gtsf2vfp.S",
                                                   "arm/ledf2vfp.S",
                                                   "arm/lesf2vfp.S",
                                                   "arm/ltdf2vfp.S",
                                                   "arm/ltsf2vfp.S",
                                                   "arm/modsi3.S",
                                                   "arm/muldf3vfp.S",
                                                   "arm/mulsf3vfp.S",
                                                   "arm/nedf2vfp.S",
                                                   "arm/negdf2vfp.S",
                                                   "arm/negsf2vfp.S",
                                                   "arm/nesf2vfp.S",
                                                   "arm/restore_vfp_d8_d15_regs.S",
                                                   "arm/save_vfp_d8_d15_regs.S",
                                                   "arm/subdf3vfp.S",
                                                   "arm/subsf3vfp.S",
                                                   "arm/switch16.S",
                                                   "arm/switch32.S",
                                                   "arm/switch8.S",
                                                   "arm/switchu8.S",
                                                   "arm/sync_fetch_and_add_4.S",
                                                   "arm/sync_fetch_and_add_8.S",
                                                   "arm/sync_fetch_and_and_4.S",
                                                   "arm/sync_fetch_and_and_8.S",
                                                   "arm/sync_fetch_and_max_4.S",
                                                   "arm/sync_fetch_and_max_8.S",
                                                   "arm/sync_fetch_and_min_4.S",
                                                   "arm/sync_fetch_and_min_8.S",
                                                   "arm/sync_fetch_and_nand_4.S",
                                                   "arm/sync_fetch_and_nand_8.S",
                                                   "arm/sync_fetch_and_or_4.S",
                                                   "arm/sync_fetch_and_or_8.S",
                                                   "arm/sync_fetch_and_sub_4.S",
                                                   "arm/sync_fetch_and_sub_8.S",
                                                   "arm/sync_fetch_and_umax_4.S",
                                                   "arm/sync_fetch_and_umax_8.S",
                                                   "arm/sync_fetch_and_umin_4.S",
                                                   "arm/sync_fetch_and_umin_8.S",
                                                   "arm/sync_fetch_and_xor_4.S",
                                                   "arm/sync_fetch_and_xor_8.S",
                                                   "arm/sync_synchronize.S",
                                                   "arm/truncdfsf2vfp.S",
                                                   "arm/udivmodsi4.S",
                                                   "arm/udivsi3.S",
                                                   "arm/umodsi3.S",
                                                   "arm/unorddf2vfp.S",
                                                   "arm/unordsf2vfp.S"];

    // NOTE These asm implementations only work in ARM mode. IOW, these don't work in THUMB mode.
    const THUMB_BLACKLIST: &'static [&'static str] = &["arm/aeabi_cdcmp.S",
                                                       "arm/aeabi_cfcmp.S",
                                                       "arm/eqdf2vfp.S",
                                                       "arm/gedf2vfp.S",
                                                       "arm/gtdf2vfp.S",
                                                       "arm/ledf2vfp.S",
                                                       "arm/ltdf2vfp.S",
                                                       "arm/ltsf2vfp.S",
                                                       "arm/nedf2vfp.S",
                                                       "arm/nesf2vfp.S",
                                                       "arm/unorddf2vfp.S",
                                                       "arm/unordsf2vfp.S"];

    const ARMV6M_BLACKLIST: &'static [&'static str] = &["arm/aeabi_dcmp.S",
                                                        "arm/aeabi_fcmp.S",
                                                        "arm/aeabi_ldivmod.S",
                                                        "arm/aeabi_uldivmod.S",
                                                        "arm/clzdi2.S",
                                                        "arm/clzsi2.S",
                                                        "arm/comparesf2.S",
                                                        "arm/divmodsi4.S",
                                                        "arm/divsi3.S",
                                                        "arm/modsi3.S",
                                                        "arm/negdf2vfp.S",
                                                        "arm/negsf2vfp.S",
                                                        "arm/switch16.S",
                                                        "arm/switch32.S",
                                                        "arm/switch8.S",
                                                        "arm/switchu8.S",
                                                        "arm/sync_fetch_and_add_4.S",
                                                        "arm/sync_fetch_and_and_4.S",
                                                        "arm/sync_fetch_and_max_4.S",
                                                        "arm/sync_fetch_and_min_4.S",
                                                        "arm/sync_fetch_and_nand_4.S",
                                                        "arm/sync_fetch_and_or_4.S",
                                                        "arm/sync_fetch_and_sub_4.S",
                                                        "arm/sync_fetch_and_umax_4.S",
                                                        "arm/sync_fetch_and_umin_4.S",
                                                        "arm/sync_fetch_and_xor_4.S",
                                                        "arm/udivmodsi4.S",
                                                        "arm/udivsi3.S",
                                                        "arm/umodsi3.S"];

    const OS_NONE_BLACKLIST: &'static [&'static str] = &["enable_execute_stack.c"];

    const SOFT_FLOAT_BLACKLIST: &'static [&'static str] = &["arm/adddf3vfp.S",
                                                            "arm/addsf3vfp.S",
                                                            "arm/divdf3vfp.S",
                                                            "arm/divsf3vfp.S",
                                                            "arm/eqdf2vfp.S",
                                                            "arm/eqsf2vfp.S",
                                                            "arm/extendsfdf2vfp.S",
                                                            "arm/fixdfsivfp.S",
                                                            "arm/fixdfsivfp.S",
                                                            "arm/fixsfsivfp.S",
                                                            "arm/fixunsdfsivfp.S",
                                                            "arm/fixunssfsivfp.S",
                                                            "arm/floatsidfvfp.S",
                                                            "arm/floatsisfvfp.S",
                                                            "arm/floatunssidfvfp.S",
                                                            "arm/floatunssisfvfp.S",
                                                            "arm/gedf2vfp.S",
                                                            "arm/gesf2vfp.S",
                                                            "arm/gtdf2vfp.S",
                                                            "arm/gtsf2vfp.S",
                                                            "arm/ledf2vfp.S",
                                                            "arm/lesf2vfp.S",
                                                            "arm/ltdf2vfp.S",
                                                            "arm/ltsf2vfp.S",
                                                            "arm/muldf3vfp.S",
                                                            "arm/mulsf3vfp.S",
                                                            "arm/nedf2vfp.S",
                                                            "arm/nesf2vfp.S",
                                                            "arm/restore_vfp_d8_d15_regs.S",
                                                            "arm/save_vfp_d8_d15_regs.S",
                                                            "arm/subdf3vfp.S",
                                                            "arm/subsf3vfp.S",
                                                            "arm/truncdfsf2vfp.S",
                                                            "arm/unorddf2vfp.S",
                                                            "arm/unordsf2vfp.S"];

    // NOTE these intrinsics require a DP FPU
    const SP_FPU_BLACKLIST: &'static [&'static str] = &["arm/adddf3vfp.S",
                                                        "arm/divdf3vfp.S",
                                                        "arm/eqsf2vfp.S",
                                                        "arm/extendsfdf2vfp.S",
                                                        "arm/fixdfsivfp.S",
                                                        "arm/fixunsdfsivfp.S",
                                                        "arm/floatsidfvfp.S",
                                                        "arm/floatunssidfvfp.S",
                                                        "arm/gesf2vfp.S",
                                                        "arm/gtsf2vfp.S",
                                                        "arm/lesf2vfp.S",
                                                        "arm/muldf3vfp.S",
                                                        "arm/subdf3vfp.S",
                                                        "arm/truncdfsf2vfp.S"];

    let mut config = Config::new();
    for source in GENERIC_SOURCES {
        if target.os_is("none") {
            if !OS_NONE_BLACKLIST.contains(source) {
                config.file(src.join("lib/builtins").join(source));
            }
        } else {
            config.file(src.join("lib/builtins").join(source));
        }
    }

    if target.arch_is("arm") {
        for source in ARM_SOURCES {
            if target.llvm_target().starts_with("thumb") && THUMB_BLACKLIST.contains(source) {
                continue
            }

            if target.llvm_target().starts_with("thumbv6m") && ARMV6M_BLACKLIST.contains(source) {
                continue
            }

            // FIXME this is wrong for Cortex-M processors, e.g. the llvm-target field of the
            // cortex-m4f target  doesn't end in hf but it does support FPU instructions.
            if !target.llvm_target().starts_with("thumbv7em") ||
                target.features().map(|f| f.contains("+soft-float")) == Some(true) ||
                target.cpu().is_none() &&
                SOFT_FLOAT_BLACKLIST.contains(source)
            {
                continue
            }

            if target.cpu() == Some("cortex-m4") && SP_FPU_BLACKLIST.contains(source) {
                continue
            }

            config.file(src.join("lib/builtins").join(source));
        }
    }

    if target.name != try!(env::var("HOST")) {
        config.archiver(Path::new(&*target.tool("AR", "ar")));
        config.compiler(Path::new(&*target.tool("CC", "gcc")));
    }

    // ARM arch optimization
    if target.arch_is("arm") {
        if target.llvm_target().contains("v6m") {
            config.flag("-march=armv6-m");
        }

        if target.llvm_target().contains("v7m") {
            config.flag("-march=armv7-m");
        }

        if target.llvm_target().contains("v7em") {
            config.flag("-march=armv7e-m");
        }
    }

    // CPU optimization
    if let Some(cpu) = target.cpu() {
        config.flag(&format!("-mcpu={}", cpu));
    }

    // THUMB mode
    if target.llvm_target().starts_with("thumb") {
        config.flag("-mthumb");
    }

    // FPU
    if target.cpu() == Some("cortex-m4") &&
        target.features().map(|f| f.contains("+soft-float")) != Some(true) {
            config.flag("-mfpu=fpv4-sp-d16");
    }

    config.compile("libcompiler-rt.a");
}

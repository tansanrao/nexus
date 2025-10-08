// This file is auto-generated from the lore.kernel.org manifest
// DO NOT EDIT MANUALLY

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailingListSeed {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub repos: Vec<RepoShard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoShard {
    pub url: String,
    pub order: i32,
}

pub fn get_all_mailing_lists() -> Vec<MailingListSeed> {
    vec![
        MailingListSeed {
            name: "Accel-Config development".to_string(),
            slug: "accel-config".to_string(),
            description: Some("Accel-Config development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/accel-config/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "ACPICA development".to_string(),
            slug: "acpica-devel".to_string(),
            description: Some("ACPICA development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/acpica-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Alsa-Devel Archive on lore.kernel.org".to_string(),
            slug: "alsa-devel".to_string(),
            description: Some("Alsa-Devel Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/alsa-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "AMD-GFX Archive on lore.kernel.org".to_string(),
            slug: "amd-gfx".to_string(),
            description: Some("AMD-GFX Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/amd-gfx/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Discuss SCMI firmware, SCMI drivers in Linux, U-boot, OP-TEE".to_string(),
            slug: "arm-scmi".to_string(),
            description: Some("Discuss SCMI firmware, SCMI drivers in Linux, U-boot, OP-TEE [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/arm-scmi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux on Apple ARM platform development".to_string(),
            slug: "asahi".to_string(),
            description: Some("Linux on Apple ARM platform development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/asahi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "ATH10K Archive on lore.kernel.org".to_string(),
            slug: "ath10k".to_string(),
            description: Some("ATH10K Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ath10k/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "ATH11K Archive on lore.kernel.org".to_string(),
            slug: "ath11k".to_string(),
            description: Some("ATH11K Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ath11k/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "($INBOX_DIR/description missing)".to_string(),
            slug: "ath12k".to_string(),
            description: Some("($INBOX_DIR/description missing) [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ath12k/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Historical ath9k-devel archives".to_string(),
            slug: "ath9k-devel".to_string(),
            description: Some("Historical ath9k-devel archives [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ath9k-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Audit system development".to_string(),
            slug: "audit".to_string(),
            description: Some("Audit system development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/audit/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "AutoFS development".to_string(),
            slug: "autofs".to_string(),
            description: Some("AutoFS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/autofs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "B4 Sent Patches".to_string(),
            slug: "b4-sent".to_string(),
            description: Some("B4 Sent Patches".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/b4-sent/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "b43-dev Archive on lore.kernel.org".to_string(),
            slug: "b43-dev".to_string(),
            description: Some("b43-dev Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/b43-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux backports project".to_string(),
            slug: "backports".to_string(),
            description: Some("Linux backports project [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/backports/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "mail archive of the barebox mailing list".to_string(),
            slug: "barebox".to_string(),
            description: Some("mail archive of the barebox mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/barebox/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "B.A.T.M.A.N Archive on lore.kernel.org".to_string(),
            slug: "batman".to_string(),
            description: Some("B.A.T.M.A.N Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/batman/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Openembedded Bitbake Development".to_string(),
            slug: "bitbake-devel".to_string(),
            description: Some("Openembedded Bitbake Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/bitbake-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "BPF List".to_string(),
            slug: "bpf".to_string(),
            description: Some("BPF List [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/bpf/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux brcm80211 wireless device drivers".to_string(),
            slug: "brcm80211".to_string(),
            description: Some("Linux brcm80211 wireless device drivers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/brcm80211/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Ethernet Bridge development".to_string(),
            slug: "bridge".to_string(),
            description: Some("Ethernet Bridge development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/bridge/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Buildroot Archive on lore.kernel.org".to_string(),
            slug: "buildroot".to_string(),
            description: Some("Buildroot Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/buildroot/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Porting Linux userspace to modern C".to_string(),
            slug: "c-std-porting".to_string(),
            description: Some("Porting Linux userspace to modern C [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/c-std-porting/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CCAN Archive on lore.kernel.org".to_string(),
            slug: "ccan".to_string(),
            description: Some("CCAN Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ccan/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CEPH filesystem development".to_string(),
            slug: "ceph-devel".to_string(),
            description: Some("CEPH filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ceph-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux cgroups development".to_string(),
            slug: "cgroups".to_string(),
            description: Some("Linux cgroups development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cgroups/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Chrome platform driver development".to_string(),
            slug: "chrome-platform".to_string(),
            description: Some("Chrome platform driver development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/chrome-platform/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CIP-dev Archive on lore.kernel.org".to_string(),
            slug: "cip-dev".to_string(),
            description: Some("CIP-dev Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cip-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Cluster-Devel Archive on lore.kernel.org".to_string(),
            slug: "cluster-devel".to_string(),
            description: Some("Cluster-Devel Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cluster-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Coccinelle Archive on lore.kernel.org".to_string(),
            slug: "cocci".to_string(),
            description: Some("Coccinelle Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cocci/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Coconut-SVSM development mailing list".to_string(),
            slug: "coconut-svsm".to_string(),
            description: Some("Coconut-SVSM development mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/coconut-svsm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "ConnMan network manager".to_string(),
            slug: "connman".to_string(),
            description: Some("ConnMan network manager [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/connman/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Container Development".to_string(),
            slug: "containers".to_string(),
            description: Some("Linux Container Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/containers/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "cpufreq Archive on lore.kernel.org".to_string(),
            slug: "cpufreq".to_string(),
            description: Some("cpufreq Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cpufreq/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CRIU (Checkpoint/Restore in Userspace) mailing list".to_string(),
            slug: "criu".to_string(),
            description: Some("CRIU (Checkpoint/Restore in Userspace) mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/criu/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Cryptsetup development".to_string(),
            slug: "cryptsetup".to_string(),
            description: Some("Cryptsetup development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cryptsetup/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CTI Technical Advisory Committee".to_string(),
            slug: "cti-tac".to_string(),
            description: Some("CTI Technical Advisory Committee [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/cti-tac/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DAMON development mailing list".to_string(),
            slug: "damon".to_string(),
            description: Some("DAMON development mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/damon/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DASH Shell discussions".to_string(),
            slug: "dash".to_string(),
            description: Some("DASH Shell discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dash/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DCCP protocol discussions".to_string(),
            slug: "dccp".to_string(),
            description: Some("DCCP protocol discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dccp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "List used for roundtrip monitoring".to_string(),
            slug: "ddprobe".to_string(),
            description: Some("List used for roundtrip monitoring [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ddprobe/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Devicetree Compiler".to_string(),
            slug: "devicetree-compiler".to_string(),
            description: Some("Devicetree Compiler [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/devicetree-compiler/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Devicetree Spec".to_string(),
            slug: "devicetree-spec".to_string(),
            description: Some("Devicetree Spec [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/devicetree-spec/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DiaMon (diagnostic and monitoring)".to_string(),
            slug: "diamon-discuss".to_string(),
            description: Some("DiaMon (diagnostic and monitoring) [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/diamon-discuss/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Forum for Linux distributions to discuss problems and share PSAs".to_string(),
            slug: "distributions".to_string(),
            description: Some("Forum for Linux distributions to discuss problems and share PSAs [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/distributions/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DM-Crypt Archive on lore.kernel.org".to_string(),
            slug: "dm-crypt".to_string(),
            description: Some("DM-Crypt Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dm-crypt/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Device Mapper development".to_string(),
            slug: "dm-devel".to_string(),
            description: Some("Linux Device Mapper development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dm-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DMA Engine development".to_string(),
            slug: "dmaengine".to_string(),
            description: Some("DMA Engine development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dmaengine/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "DPDK-dev Archive on lore.kernel.org".to_string(),
            slug: "dpdk-dev".to_string(),
            description: Some("DPDK-dev Archive on lore.kernel.org [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dpdk-dev/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/dpdk-dev/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "dri-devel Archive on lore.kernel.org".to_string(),
            slug: "dri-devel".to_string(),
            description: Some("dri-devel Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dri-devel/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/dri-devel/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "Linux DTrace development list".to_string(),
            slug: "dtrace".to_string(),
            description: Some("Linux DTrace development list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dtrace/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Dwarves debugging tools".to_string(),
            slug: "dwarves".to_string(),
            description: Some("Dwarves debugging tools [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/dwarves/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "EcryptFS development".to_string(),
            slug: "ecryptfs".to_string(),
            description: Some("EcryptFS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ecryptfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Embedded Linux Library".to_string(),
            slug: "ell".to_string(),
            description: Some("Embedded Linux Library [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ell/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Flexible I/O Tester development".to_string(),
            slug: "fio".to_string(),
            description: Some("Flexible I/O Tester development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/fio/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "FS/XFS testing framework".to_string(),
            slug: "fstests".to_string(),
            description: Some("FS/XFS testing framework [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/fstests/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux fsverity development list".to_string(),
            slug: "fsverity".to_string(),
            description: Some("Linux fsverity development list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/fsverity/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Fuego test framework".to_string(),
            slug: "fuego".to_string(),
            description: Some("Fuego test framework [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/fuego/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "gfs2 filesystem and dlm development".to_string(),
            slug: "gfs2".to_string(),
            description: Some("gfs2 filesystem and dlm development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/gfs2/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Git development".to_string(),
            slug: "git".to_string(),
            description: Some("Git development [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/git/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/git/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "Grub Development Archive on lore.kernel.org".to_string(),
            slug: "grub-devel".to_string(),
            description: Some("Grub Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/grub-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Hail cloud computing Development Archive on lore.kernel.org".to_string(),
            slug: "hail-devel".to_string(),
            description: Some("Hail cloud computing Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/hail-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Historical speck list archives".to_string(),
            slug: "historical-speck".to_string(),
            description: Some("Historical speck list archives [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/historical-speck/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Igt-dev Archive on lore.kernel.org".to_string(),
            slug: "igt-dev".to_string(),
            description: Some("Igt-dev Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/igt-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel and device drivers for NXP i.MX platforms".to_string(),
            slug: "imx".to_string(),
            description: Some("Linux kernel and device drivers for NXP i.MX platforms [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/imx/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "mkinitrd unification across distributions".to_string(),
            slug: "initramfs".to_string(),
            description: Some("mkinitrd unification across distributions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/initramfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Intel-GFX Archive on lore.kernel.org".to_string(),
            slug: "intel-gfx".to_string(),
            description: Some("Intel-GFX Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/intel-gfx/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/intel-gfx/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "Intel-Wired-Lan Archive on lore.kernel.org".to_string(),
            slug: "intel-wired-lan".to_string(),
            description: Some("Intel-Wired-Lan Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/intel-wired-lan/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Intel-XE Archive on lore.kernel.org".to_string(),
            slug: "intel-xe".to_string(),
            description: Some("Intel-XE Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/intel-xe/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux io-uring development".to_string(),
            slug: "io-uring".to_string(),
            description: Some("Linux io-uring development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/io-uring/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Wireless Daemon for Linux".to_string(),
            slug: "iwd".to_string(),
            description: Some("Wireless Daemon for Linux [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/iwd/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux console tools development".to_string(),
            slug: "kbd".to_string(),
            description: Some("Linux console tools development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kbd/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel kdevops project".to_string(),
            slug: "kdevops".to_string(),
            description: Some("Linux kernel kdevops project [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kdevops/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel-hardening Archive on lore.kernel.org".to_string(),
            slug: "kernel-hardening".to_string(),
            description: Some("Kernel-hardening Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernel-hardening/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel development cleanups and similar janitorial tasks".to_string(),
            slug: "kernel-janitors".to_string(),
            description: Some("Kernel development cleanups and similar janitorial tasks [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernel-janitors/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel-testers Development Archive on lore.kernel.org".to_string(),
            slug: "kernel-testers".to_string(),
            description: Some("Kernel-testers Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernel-testers/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Handshake for in-kernel TLS consumers".to_string(),
            slug: "kernel-tls-handshake".to_string(),
            description: Some("Handshake for in-kernel TLS consumers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernel-tls-handshake/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "KernelCI discussions".to_string(),
            slug: "kernelci".to_string(),
            description: Some("KernelCI discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernelci/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel Newbies Archive on lore.kernel.org".to_string(),
            slug: "kernelnewbies".to_string(),
            description: Some("Kernel Newbies Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kernelnewbies/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kexec Archive on lore.kernel.org".to_string(),
            slug: "kexec".to_string(),
            description: Some("Kexec Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kexec/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel keyring development".to_string(),
            slug: "keyrings".to_string(),
            description: Some("Kernel keyring development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/keyrings/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Submissions of public keys to the kernel.org web of trust".to_string(),
            slug: "keys".to_string(),
            description: Some("Submissions of public keys to the kernel.org web of trust [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/keys/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Kernel Summit discussions".to_string(),
            slug: "ksummit".to_string(),
            description: Some("Linux Kernel Summit discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ksummit/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel KVM virtualization development".to_string(),
            slug: "kvm".to_string(),
            description: Some("Kernel KVM virtualization development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kvm/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/kvm/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "Kvm-ia64 Development Archive on lore.kernel.org".to_string(),
            slug: "kvm-ia64".to_string(),
            description: Some("Kvm-ia64 Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kvm-ia64/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Kernel KVM-PPC virtualization development".to_string(),
            slug: "kvm-ppc".to_string(),
            description: Some("Kernel KVM-PPC virtualization development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kvm-ppc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "KVM-RISCV Archive on lore.kernel.org".to_string(),
            slug: "kvm-riscv".to_string(),
            description: Some("KVM-RISCV Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kvm-riscv/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux KVM/arm64 development list".to_string(),
            slug: "kvmarm".to_string(),
            description: Some("Linux KVM/arm64 development list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/kvmarm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Landlock LSM user space discussions".to_string(),
            slug: "landlock".to_string(),
            description: Some("Landlock LSM user space discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/landlock/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Advanced Routing and Traffic Control list".to_string(),
            slug: "lartc".to_string(),
            description: Some("Linux Advanced Routing and Traffic Control list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lartc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux CH - Switzerland".to_string(),
            slug: "lch".to_string(),
            description: Some("Linux CH - Switzerland [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lch/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "String-to-numeric library".to_string(),
            slug: "liba2i".to_string(),
            description: Some("String-to-numeric library [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/liba2i/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-8086 Development Archive on lore.kernel.org".to_string(),
            slug: "linux-8086".to_string(),
            description: Some("Linux-8086 Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-8086/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux ACPI".to_string(),
            slug: "linux-acpi".to_string(),
            description: Some("Linux ACPI [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-acpi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-admin Development Archive on lore.kernel.org".to_string(),
            slug: "linux-admin".to_string(),
            description: Some("Linux-admin Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-admin/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Alpha arch development list".to_string(),
            slug: "linux-alpha".to_string(),
            description: Some("Alpha arch development list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-alpha/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Amlogic Archive on lore.kernel.org".to_string(),
            slug: "linux-amlogic".to_string(),
            description: Some("Linux-Amlogic Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-amlogic/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux userland API discussions".to_string(),
            slug: "linux-api".to_string(),
            description: Some("Linux userland API discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-api/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Generic Linux architectural discussions".to_string(),
            slug: "linux-arch".to_string(),
            description: Some("Generic Linux architectural discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-arch/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-ARM-Kernel Archive on lore.kernel.org".to_string(),
            slug: "linux-arm-kernel".to_string(),
            description: Some("Linux-ARM-Kernel Archive on lore.kernel.org [epoch 2]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-arm-kernel/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/linux-arm-kernel/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/linux-arm-kernel/git/2.git".to_string(), order: 2 },
                RepoShard { url: "https://lore.kernel.org/linux-arm-kernel/git/3.git".to_string(), order: 3 },
            ],
        },
        MailingListSeed {
            name: "Linux ARM-MSM sub-architecture".to_string(),
            slug: "linux-arm-msm".to_string(),
            description: Some("Linux ARM-MSM sub-architecture [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-arm-msm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Aspeed Archive on lore.kernel.org".to_string(),
            slug: "linux-aspeed".to_string(),
            description: Some("Linux-Aspeed Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-aspeed/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux assembly list".to_string(),
            slug: "linux-assembly".to_string(),
            description: Some("Linux assembly list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-assembly/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-audit Archive on lore.kernel.org".to_string(),
            slug: "linux-audit".to_string(),
            description: Some("Linux-audit Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-audit/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux bcache driver list".to_string(),
            slug: "linux-bcache".to_string(),
            description: Some("Linux bcache driver list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-bcache/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux bcachefs list".to_string(),
            slug: "linux-bcachefs".to_string(),
            description: Some("Linux bcachefs list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-bcachefs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux block layer".to_string(),
            slug: "linux-block".to_string(),
            description: Some("Linux block layer [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-block/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux bluetooth development".to_string(),
            slug: "linux-bluetooth".to_string(),
            description: Some("Linux bluetooth development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-bluetooth/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux btrace development".to_string(),
            slug: "linux-btrace".to_string(),
            description: Some("Linux btrace development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-btrace/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Btrfs filesystem development".to_string(),
            slug: "linux-btrfs".to_string(),
            description: Some("Linux Btrfs filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-btrfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Generic list for bug discussions".to_string(),
            slug: "linux-bugs".to_string(),
            description: Some("Generic list for bug discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-bugs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-c-programming Development Archive on lore.kernel.org".to_string(),
            slug: "linux-c-programming".to_string(),
            description: Some("Linux-c-programming Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-c-programming/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux CAN drivers development".to_string(),
            slug: "linux-can".to_string(),
            description: Some("Linux CAN drivers development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-can/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux CIFS filesystem development".to_string(),
            slug: "linux-cifs".to_string(),
            description: Some("Linux CIFS filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-cifs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux clock framework development".to_string(),
            slug: "linux-clk".to_string(),
            description: Some("Linux clock framework development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-clk/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Confidential Computing Development".to_string(),
            slug: "linux-coco".to_string(),
            description: Some("Linux Confidential Computing Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-coco/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-config Development Archive on lore.kernel.org".to_string(),
            slug: "linux-config".to_string(),
            description: Some("Linux-config Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-config/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-console Development Archive on lore.kernel.org".to_string(),
            slug: "linux-console".to_string(),
            description: Some("Linux-console Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-console/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux cryptographic layer development".to_string(),
            slug: "linux-crypto".to_string(),
            description: Some("Linux cryptographic layer development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-crypto/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux C-SKY architecture development".to_string(),
            slug: "linux-csky".to_string(),
            description: Some("Linux C-SKY architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-csky/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel CVE announcements".to_string(),
            slug: "linux-cve-announce".to_string(),
            description: Some("Linux kernel CVE announcements [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-cve-announce/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux CXL".to_string(),
            slug: "linux-cxl".to_string(),
            description: Some("Linux CXL [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-cxl/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux debuggers".to_string(),
            slug: "linux-debuggers".to_string(),
            description: Some("Linux debuggers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-debuggers/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Devicetree".to_string(),
            slug: "linux-devicetree".to_string(),
            description: Some("Devicetree [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-devicetree/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/linux-devicetree/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/linux-devicetree/git/2.git".to_string(), order: 2 },
            ],
        },
        MailingListSeed {
            name: "Linux-diald Development Archive on lore.kernel.org".to_string(),
            slug: "linux-diald".to_string(),
            description: Some("Linux-diald Development Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-diald/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Documentation".to_string(),
            slug: "linux-doc".to_string(),
            description: Some("Linux Documentation [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-doc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux EDAC development".to_string(),
            slug: "linux-edac".to_string(),
            description: Some("Linux EDAC development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-edac/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux EFI development".to_string(),
            slug: "linux-efi".to_string(),
            description: Some("Linux EFI development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-efi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Embedded Linux development".to_string(),
            slug: "linux-embedded".to_string(),
            description: Some("Embedded Linux development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-embedded/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-EROFS Archive on lore.kernel.org".to_string(),
            slug: "linux-erofs".to_string(),
            description: Some("Linux-EROFS Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-erofs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux EXT4 FS development".to_string(),
            slug: "linux-ext4".to_string(),
            description: Some("Linux EXT4 FS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-ext4/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-f2fs-devel Archive on lore.kernel.org".to_string(),
            slug: "linux-f2fs-devel".to_string(),
            description: Some("Linux-f2fs-devel Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-f2fs-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Framebuffer Layer development".to_string(),
            slug: "linux-fbdev".to_string(),
            description: Some("Linux Framebuffer Layer development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-fbdev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Firmware Archive on lore.kernel.org".to_string(),
            slug: "linux-firmware".to_string(),
            description: Some("Linux-Firmware Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-firmware/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux FPGA development".to_string(),
            slug: "linux-fpga".to_string(),
            description: Some("Linux FPGA development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-fpga/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux FSCRYPT development".to_string(),
            slug: "linux-fscrypt".to_string(),
            description: Some("Linux FSCRYPT development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-fscrypt/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux filesystem development".to_string(),
            slug: "linux-fsdevel".to_string(),
            description: Some("Linux filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-fsdevel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux GCC discussions".to_string(),
            slug: "linux-gcc".to_string(),
            description: Some("Linux GCC discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-gcc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux GPIO subsystem development".to_string(),
            slug: "linux-gpio".to_string(),
            description: Some("Linux GPIO subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-gpio/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux HAM/Amateur Radio development".to_string(),
            slug: "linux-hams".to_string(),
            description: Some("Linux HAM/Amateur Radio development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hams/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Hardening".to_string(),
            slug: "linux-hardening".to_string(),
            description: Some("Linux Hardening [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hardening/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Hexagon architecture development".to_string(),
            slug: "linux-hexagon".to_string(),
            description: Some("Linux Hexagon architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hexagon/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Hotplug development".to_string(),
            slug: "linux-hotplug".to_string(),
            description: Some("Linux Hotplug development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hotplug/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Hardware Monitor development".to_string(),
            slug: "linux-hwmon".to_string(),
            description: Some("Linux Hardware Monitor development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hwmon/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-HyperV List".to_string(),
            slug: "linux-hyperv".to_string(),
            description: Some("Linux-HyperV List [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-hyperv/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux I2C development".to_string(),
            slug: "linux-i2c".to_string(),
            description: Some("Linux I2C development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-i2c/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-i3c Archive on lore.kernel.org".to_string(),
            slug: "linux-i3c".to_string(),
            description: Some("Linux-i3c Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-i3c/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux IA64 platform development".to_string(),
            slug: "linux-ia64".to_string(),
            description: Some("Linux IA64 platform development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-ia64/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux ATA/IDE development".to_string(),
            slug: "linux-ide".to_string(),
            description: Some("Linux ATA/IDE development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-ide/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux IIO development".to_string(),
            slug: "linux-iio".to_string(),
            description: Some("Linux IIO development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-iio/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Input/HID development".to_string(),
            slug: "linux-input".to_string(),
            description: Some("Linux Input/HID development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-input/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Integrity Measurement development".to_string(),
            slug: "linux-integrity".to_string(),
            description: Some("Linux Integrity Measurement development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-integrity/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux IOMMU Development".to_string(),
            slug: "linux-iommu".to_string(),
            description: Some("Linux IOMMU Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-iommu/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kbuild/kconfig development".to_string(),
            slug: "linux-kbuild".to_string(),
            description: Some("Linux kbuild/kconfig development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-kbuild/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel announcements".to_string(),
            slug: "linux-kernel-announce".to_string(),
            description: Some("Linux kernel announcements [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-kernel-announce/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Kernel Mentees list".to_string(),
            slug: "linux-kernel-mentees".to_string(),
            description: Some("Linux Kernel Mentees list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-kernel-mentees/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Kernel Selftest development".to_string(),
            slug: "linux-kselftest".to_string(),
            description: Some("Linux Kernel Selftest development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-kselftest/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux laptop Discussion Archive on lore.kernel.org".to_string(),
            slug: "linux-laptop".to_string(),
            description: Some("Linux laptop Discussion Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-laptop/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux LED subsystem development".to_string(),
            slug: "linux-leds".to_string(),
            description: Some("Linux LED subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-leds/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux LVM users".to_string(),
            slug: "linux-lvm".to_string(),
            description: Some("Linux LVM users [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-lvm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux M68K Architecture development".to_string(),
            slug: "linux-m68k".to_string(),
            description: Some("Linux M68K Architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-m68k/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Manual Pages development".to_string(),
            slug: "linux-man".to_string(),
            description: Some("Linux Manual Pages development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-man/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Media Controller development".to_string(),
            slug: "linux-media".to_string(),
            description: Some("Linux Media Controller development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-media/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-mediatek Archive on lore.kernel.org".to_string(),
            slug: "linux-mediatek".to_string(),
            description: Some("Linux-mediatek Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-mediatek/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Metag architecture Discussion Archive on lore.kernel.org".to_string(),
            slug: "linux-metag".to_string(),
            description: Some("Linux Metag architecture Discussion Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-metag/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux MIPS Architecture development".to_string(),
            slug: "linux-mips".to_string(),
            description: Some("Linux MIPS Architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-mips/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-mm Archive on lore.kernel.org".to_string(),
            slug: "linux-mm".to_string(),
            description: Some("Linux-mm Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-mm/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/linux-mm/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/linux-mm/git/2.git".to_string(), order: 2 },
            ],
        },
        MailingListSeed {
            name: "Linux MultiMedia Card development".to_string(),
            slug: "linux-mmc".to_string(),
            description: Some("Linux MultiMedia Card development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-mmc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Modules".to_string(),
            slug: "linux-modules".to_string(),
            description: Some("Linux Modules [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-modules/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux MS DOS discussions".to_string(),
            slug: "linux-msdos".to_string(),
            description: Some("Linux MS DOS discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-msdos/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-mtd Archive on lore.kernel.org".to_string(),
            slug: "linux-mtd".to_string(),
            description: Some("Linux-mtd Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-mtd/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Newbie help".to_string(),
            slug: "linux-newbie".to_string(),
            description: Some("Linux Newbie help [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-newbie/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Next discussions".to_string(),
            slug: "linux-next".to_string(),
            description: Some("Linux-Next discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-next/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-NFC Archive on lore.kernel.org".to_string(),
            slug: "linux-nfc".to_string(),
            description: Some("Linux-NFC Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-nfc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux NFS development".to_string(),
            slug: "linux-nfs".to_string(),
            description: Some("Linux NFS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-nfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux NILFS development".to_string(),
            slug: "linux-nilfs".to_string(),
            description: Some("Linux NILFS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-nilfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux NUMA userland tools development".to_string(),
            slug: "linux-numa".to_string(),
            description: Some("Linux NUMA userland tools development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-numa/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-NVDIMM Archive on lore.kernel.org".to_string(),
            slug: "linux-nvdimm".to_string(),
            description: Some("Linux-NVDIMM Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-nvdimm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-NVME Archive on lore.kernel.org".to_string(),
            slug: "linux-nvme".to_string(),
            description: Some("Linux-NVME Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-nvme/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux on ARM based TI OMAP SoCs".to_string(),
            slug: "linux-omap".to_string(),
            description: Some("Linux on ARM based TI OMAP SoCs [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-omap/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "linux-oxnas archives".to_string(),
            slug: "linux-oxnas".to_string(),
            description: Some("linux-oxnas archives [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-oxnas/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux PARISC architecture development".to_string(),
            slug: "linux-parisc".to_string(),
            description: Some("Linux PARISC architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-parisc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Archive-only list for patches".to_string(),
            slug: "linux-patches".to_string(),
            description: Some("Archive-only list for patches [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-patches/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux PCI subsystem development".to_string(),
            slug: "linux-pci".to_string(),
            description: Some("Linux PCI subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-pci/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Perf Users".to_string(),
            slug: "linux-perf-users".to_string(),
            description: Some("Linux Perf Users [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-perf-users/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-PHY Archive on lore.kernel.org".to_string(),
            slug: "linux-phy".to_string(),
            description: Some("Linux-PHY Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-phy/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Power Management development".to_string(),
            slug: "linux-pm".to_string(),
            description: Some("Linux Power Management development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-pm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux PPP protocol development".to_string(),
            slug: "linux-ppp".to_string(),
            description: Some("Linux PPP protocol development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-ppp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux PWM subsystem development".to_string(),
            slug: "linux-pwm".to_string(),
            description: Some("Linux PWM subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-pwm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux RAID subsystem development".to_string(),
            slug: "linux-raid".to_string(),
            description: Some("Linux RAID subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-raid/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux RDMA and InfiniBand development".to_string(),
            slug: "linux-rdma".to_string(),
            description: Some("Linux RDMA and InfiniBand development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-rdma/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Remote Processor Subsystem development".to_string(),
            slug: "linux-remoteproc".to_string(),
            description: Some("Linux Remote Processor Subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-remoteproc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Renesas SOC kernel development".to_string(),
            slug: "linux-renesas-soc".to_string(),
            description: Some("Linux Renesas SOC kernel development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-renesas-soc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-RISC-V Archive on lore.kernel.org".to_string(),
            slug: "linux-riscv".to_string(),
            description: Some("Linux-RISC-V Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-riscv/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Rockchip Archive on lore.kernel.org".to_string(),
            slug: "linux-rockchip".to_string(),
            description: Some("Linux-Rockchip Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-rockchip/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux real-time development".to_string(),
            slug: "linux-rt-devel".to_string(),
            description: Some("Linux real-time development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-rt-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux RT Users".to_string(),
            slug: "linux-rt-users".to_string(),
            description: Some("Linux RT Users [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-rt-users/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux RTC".to_string(),
            slug: "linux-rtc".to_string(),
            description: Some("Linux RTC [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-rtc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux s390 Architecture development".to_string(),
            slug: "linux-s390".to_string(),
            description: Some("Linux s390 Architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-s390/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux-Safety Archive on lore.kernel.org".to_string(),
            slug: "linux-safety".to_string(),
            description: Some("Linux-Safety Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-safety/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Samsung SOC development".to_string(),
            slug: "linux-samsung-soc".to_string(),
            description: Some("Linux Samsung SOC development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-samsung-soc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SCSI subsystem development".to_string(),
            slug: "linux-scsi".to_string(),
            description: Some("Linux SCSI subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-scsi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SCTP protocol development".to_string(),
            slug: "linux-sctp".to_string(),
            description: Some("Linux SCTP protocol development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sctp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Security Modules development".to_string(),
            slug: "linux-security-module".to_string(),
            description: Some("Linux Security Modules development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-security-module/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Serial subsystem development".to_string(),
            slug: "linux-serial".to_string(),
            description: Some("Linux Serial subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-serial/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Intel SGX development".to_string(),
            slug: "linux-sgx".to_string(),
            description: Some("Intel SGX development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sgx/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SUPERH platform development".to_string(),
            slug: "linux-sh".to_string(),
            description: Some("SUPERH platform development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sh/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Symmetric Multiprocessing (SMP) development".to_string(),
            slug: "linux-smp".to_string(),
            description: Some("Symmetric Multiprocessing (SMP) development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-smp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SNPS ARC Archive on lore.kernel.org".to_string(),
            slug: "linux-snps-arc".to_string(),
            description: Some("Linux SNPS ARC Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-snps-arc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Sound subsystem development".to_string(),
            slug: "linux-sound".to_string(),
            description: Some("Linux Sound subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sound/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SPARSE checker discussions".to_string(),
            slug: "linux-sparse".to_string(),
            description: Some("Linux SPARSE checker discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sparse/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Licenses and SPDX discussions".to_string(),
            slug: "linux-spdx".to_string(),
            description: Some("Linux Licenses and SPDX discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-spdx/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SPI subsystem development".to_string(),
            slug: "linux-spi".to_string(),
            description: Some("Linux SPI subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-spi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel staging patches".to_string(),
            slug: "linux-staging".to_string(),
            description: Some("Linux kernel staging patches [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-staging/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "ARM Sunxi Platform Development".to_string(),
            slug: "linux-sunxi".to_string(),
            description: Some("ARM Sunxi Platform Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-sunxi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Tegra architecture development".to_string(),
            slug: "linux-tegra".to_string(),
            description: Some("Linux Tegra architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-tegra/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux toolchain discussions".to_string(),
            slug: "linux-toolchains".to_string(),
            description: Some("Linux toolchain discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-toolchains/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Trace Development".to_string(),
            slug: "linux-trace-devel".to_string(),
            description: Some("Linux Trace Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-trace-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Trace Kernel".to_string(),
            slug: "linux-trace-kernel".to_string(),
            description: Some("Linux Trace Kernel [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-trace-kernel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Trace Users".to_string(),
            slug: "linux-trace-users".to_string(),
            description: Some("Linux Trace Users [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-trace-users/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "linux-um archives".to_string(),
            slug: "linux-um".to_string(),
            description: Some("linux-um archives [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-um/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Overlay Filesystem development".to_string(),
            slug: "linux-unionfs".to_string(),
            description: Some("Linux Overlay Filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-unionfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux USB".to_string(),
            slug: "linux-usb".to_string(),
            description: Some("Linux USB [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-usb/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Watchdog driver development".to_string(),
            slug: "linux-watchdog".to_string(),
            description: Some("Linux Watchdog driver development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-watchdog/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux wireless drivers development".to_string(),
            slug: "linux-wireless".to_string(),
            description: Some("Linux wireless drivers development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-wireless/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux IEEE 802.15.4 and 6LoWPAN development".to_string(),
            slug: "linux-wpan".to_string(),
            description: Some("Linux IEEE 802.15.4 and 6LoWPAN development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-wpan/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux X11 Discussion Archive on lore.kernel.org".to_string(),
            slug: "linux-x11".to_string(),
            description: Some("Linux X11 Discussion Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-x11/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux X.25 subsystem development".to_string(),
            slug: "linux-x25".to_string(),
            description: Some("Linux X.25 subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-x25/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux XFS filesystem development".to_string(),
            slug: "linux-xfs".to_string(),
            description: Some("Linux XFS filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linux-xfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "LinuxPPC-Dev Archive on lore.kernel.org".to_string(),
            slug: "linuxppc-dev".to_string(),
            description: Some("LinuxPPC-Dev Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/linuxppc-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Live Patching".to_string(),
            slug: "live-patching".to_string(),
            description: Some("Live Patching [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/live-patching/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "The Linux Kernel Mailing List".to_string(),
            slug: "lkml".to_string(),
            description: Some("The Linux Kernel Mailing List [epoch 5]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lkml/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/2.git".to_string(), order: 2 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/3.git".to_string(), order: 3 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/4.git".to_string(), order: 4 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/5.git".to_string(), order: 5 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/6.git".to_string(), order: 6 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/7.git".to_string(), order: 7 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/8.git".to_string(), order: 8 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/9.git".to_string(), order: 9 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/10.git".to_string(), order: 10 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/11.git".to_string(), order: 11 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/12.git".to_string(), order: 12 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/13.git".to_string(), order: 13 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/14.git".to_string(), order: 14 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/15.git".to_string(), order: 15 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/16.git".to_string(), order: 16 },
                RepoShard { url: "https://lore.kernel.org/lkml/git/17.git".to_string(), order: 17 },
            ],
        },
        MailingListSeed {
            name: "Linux Kernel Memory Consistency Model".to_string(),
            slug: "lkmm".to_string(),
            description: Some("Linux Kernel Memory Consistency Model [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lkmm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Building the Linux kernel with Clang and LLVM".to_string(),
            slug: "llvm".to_string(),
            description: Some("Building the Linux kernel with Clang and LLVM [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/llvm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "LM Sensors development".to_string(),
            slug: "lm-sensors".to_string(),
            description: Some("LM Sensors development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lm-sensors/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "LoongArch architecture development".to_string(),
            slug: "loongarch".to_string(),
            description: Some("LoongArch architecture development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/loongarch/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Test Project".to_string(),
            slug: "ltp".to_string(),
            description: Some("Linux Test Project [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ltp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "lttng-dev Archive on lore.kernel.org".to_string(),
            slug: "lttng-dev".to_string(),
            description: Some("lttng-dev Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lttng-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Lustre-devel archive on lore.kernel.org".to_string(),
            slug: "lustre-devel".to_string(),
            description: Some("Lustre-devel archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lustre-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Vendor Firmware Service (LVFS) announcements".to_string(),
            slug: "lvfs-announce".to_string(),
            description: Some("Linux Vendor Firmware Service (LVFS) announcements [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lvfs-announce/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Vendor Firmware Service (LVFS) development".to_string(),
            slug: "lvfs-general".to_string(),
            description: Some("Linux Vendor Firmware Service (LVFS) development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lvfs-general/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux LVM developers".to_string(),
            slug: "lvm-devel".to_string(),
            description: Some("Linux LVM developers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lvm-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "LVS and IPVS development".to_string(),
            slug: "lvs-devel".to_string(),
            description: Some("LVS and IPVS development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/lvs-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux mailbox device drivers".to_string(),
            slug: "mailbox".to_string(),
            description: Some("Linux mailbox device drivers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/mailbox/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux MHI Development".to_string(),
            slug: "mhi".to_string(),
            description: Some("Linux MHI Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/mhi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "MLMMJ Mailing List Manager".to_string(),
            slug: "mlmmj".to_string(),
            description: Some("MLMMJ Mailing List Manager [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/mlmmj/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux MM tree latest commits".to_string(),
            slug: "mm-commits".to_string(),
            description: Some("Linux MM tree latest commits [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/mm-commits/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "MPTCP Linux Development".to_string(),
            slug: "mptcp".to_string(),
            description: Some("MPTCP Linux Development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/mptcp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Multikernel Architecture for Linux Kernel".to_string(),
            slug: "multikernel".to_string(),
            description: Some("Multikernel Architecture for Linux Kernel [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/multikernel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Netdev List".to_string(),
            slug: "netdev".to_string(),
            description: Some("Netdev List [epoch 3]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/netdev/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/netdev/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/netdev/git/2.git".to_string(), order: 2 },
                RepoShard { url: "https://lore.kernel.org/netdev/git/3.git".to_string(), order: 3 },
            ],
        },
        MailingListSeed {
            name: "Linux Netfilter discussions".to_string(),
            slug: "netfilter".to_string(),
            description: Some("Linux Netfilter discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/netfilter/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Netfilter development".to_string(),
            slug: "netfilter-devel".to_string(),
            description: Some("Linux Netfilter development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/netfilter-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux network filesystem support library".to_string(),
            slug: "netfs".to_string(),
            description: Some("Linux network filesystem support library [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/netfs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "nil-migration development mailing list".to_string(),
            slug: "nil-migration".to_string(),
            description: Some("nil-migration development mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/nil-migration/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Nouveau Archive on lore.kernel.org".to_string(),
            slug: "nouveau".to_string(),
            description: Some("Nouveau Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/nouveau/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux PCI Non-Transparent Bridge framework and drivers".to_string(),
            slug: "ntb".to_string(),
            description: Some("Linux PCI Non-Transparent Bridge framework and drivers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ntb/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "NTFS3 file system kernel mode driver".to_string(),
            slug: "ntfs3".to_string(),
            description: Some("NTFS3 file system kernel mode driver [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ntfs3/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "NVDIMM Device and Persistent Memory development".to_string(),
            slug: "nvdimm".to_string(),
            description: Some("NVDIMM Device and Persistent Memory development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/nvdimm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux ocfs2 filesystem development".to_string(),
            slug: "ocfs2-devel".to_string(),
            description: Some("Linux ocfs2 filesystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ocfs2-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "CHIPSEC Discussion List".to_string(),
            slug: "oe-chipsec".to_string(),
            description: Some("CHIPSEC Discussion List [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/oe-chipsec/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "0 day kernel build service".to_string(),
            slug: "oe-kbuild".to_string(),
            description: Some("0 day kernel build service [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/oe-kbuild/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "kbuild Development List".to_string(),
            slug: "oe-kbuild-all".to_string(),
            description: Some("kbuild Development List [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/oe-kbuild-all/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/oe-kbuild-all/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "NFC on Linux".to_string(),
            slug: "oe-linux-nfc".to_string(),
            description: Some("NFC on Linux [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/oe-linux-nfc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Kernel Performance".to_string(),
            slug: "oe-lkp".to_string(),
            description: Some("Linux Kernel Performance [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/oe-lkp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Open Source Telephony".to_string(),
            slug: "ofono".to_string(),
            description: Some("Open Source Telephony [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ofono/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "OP-TEE Archive on lore.kernel.org".to_string(),
            slug: "op-tee".to_string(),
            description: Some("OP-TEE Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/op-tee/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Open Programmable Acceleration Engine".to_string(),
            slug: "opae".to_string(),
            description: Some("Open Programmable Acceleration Engine [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/opae/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Openbmc Archive on lore.kernel.org".to_string(),
            slug: "openbmc".to_string(),
            description: Some("Openbmc Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/openbmc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Openembedded Core Discussions".to_string(),
            slug: "openembedded-core".to_string(),
            description: Some("Openembedded Core Discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/openembedded-core/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Openembedded Devel Discussions".to_string(),
            slug: "openembedded-devel".to_string(),
            description: Some("Openembedded Devel Discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/openembedded-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux OpenRISC platform development".to_string(),
            slug: "openrisc".to_string(),
            description: Some("Linux OpenRISC platform development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/openrisc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "OpenSBI Archive on lore.kernel.org".to_string(),
            slug: "opensbi".to_string(),
            description: Some("OpenSBI Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/opensbi/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Outreachy Linux Kernel Community".to_string(),
            slug: "outreachy".to_string(),
            description: Some("Outreachy Linux Kernel Community [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/outreachy/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Discussions of the Parallel Programming book".to_string(),
            slug: "perfbook".to_string(),
            description: Some("Discussions of the Parallel Programming book [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/perfbook/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux smartphone development".to_string(),
            slug: "phone-devel".to_string(),
            description: Some("Linux smartphone development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/phone-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "X86 platform drivers".to_string(),
            slug: "platform-driver-x86".to_string(),
            description: Some("X86 platform drivers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/platform-driver-x86/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Poky Project Discussions".to_string(),
            slug: "poky".to_string(),
            description: Some("Poky Project Discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/poky/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "powertop - power analysis tool".to_string(),
            slug: "powertop".to_string(),
            description: Some("powertop - power analysis tool [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/powertop/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Printing architecture under Linux".to_string(),
            slug: "printing-architecture".to_string(),
            description: Some("Printing architecture under Linux [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/printing-architecture/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux printing list for end users to discuss printing issues and potential feature requests".to_string(),
            slug: "printing-users".to_string(),
            description: Some("Linux printing list for end users to discuss printing issues and potential feature requests [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/printing-users/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "QEMU-Arm Archive on lore.kernel.org".to_string(),
            slug: "qemu-arm".to_string(),
            description: Some("QEMU-Arm Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/qemu-arm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "QEMU-Devel Archive on lore.kernel.org".to_string(),
            slug: "qemu-devel".to_string(),
            description: Some("QEMU-Devel Archive on lore.kernel.org [epoch 3]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/qemu-devel/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/qemu-devel/git/1.git".to_string(), order: 1 },
                RepoShard { url: "https://lore.kernel.org/qemu-devel/git/2.git".to_string(), order: 2 },
                RepoShard { url: "https://lore.kernel.org/qemu-devel/git/3.git".to_string(), order: 3 },
            ],
        },
        MailingListSeed {
            name: "QEMU-Riscv Archive on lore.kernel.org".to_string(),
            slug: "qemu-riscv".to_string(),
            description: Some("QEMU-Riscv Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/qemu-riscv/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "QEMU-Rust Archive on lore.kernel.org".to_string(),
            slug: "qemu-rust".to_string(),
            description: Some("QEMU-Rust Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/qemu-rust/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "QEMU-Trivial Archive on lore.kernel.org".to_string(),
            slug: "qemu-trivial".to_string(),
            description: Some("QEMU-Trivial Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/qemu-trivial/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux QUIC protocol development".to_string(),
            slug: "quic".to_string(),
            description: Some("Linux QUIC protocol development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/quic/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "RadioTap Archive on lore.kernel.org".to_string(),
            slug: "radiotap".to_string(),
            description: Some("RadioTap Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/radiotap/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux RCU subsystem development".to_string(),
            slug: "rcu".to_string(),
            description: Some("Linux RCU subsystem development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/rcu/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel regressions".to_string(),
            slug: "regressions".to_string(),
            description: Some("Linux kernel regressions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/regressions/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux ReiserFS filesystem (obsolete)".to_string(),
            slug: "reiserfs-devel".to_string(),
            description: Some("Linux ReiserFS filesystem (obsolete) [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/reiserfs-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Rust for Linux List".to_string(),
            slug: "rust-for-linux".to_string(),
            description: Some("Rust for Linux List [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/rust-for-linux/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Sched_ext development".to_string(),
            slug: "sched-ext".to_string(),
            description: Some("Sched_ext development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/sched-ext/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SELinux Security Module development".to_string(),
            slug: "selinux".to_string(),
            description: Some("SELinux Security Module development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/selinux/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SELinux Reference Policy development".to_string(),
            slug: "selinux-refpolicy".to_string(),
            description: Some("SELinux Reference Policy development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/selinux-refpolicy/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Signatures Archive on lore.kernel.org".to_string(),
            slug: "signatures".to_string(),
            description: Some("Signatures Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/signatures/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Smatch (Semantic Matching Tool) development".to_string(),
            slug: "smatch".to_string(),
            description: Some("Smatch (Semantic Matching Tool) development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/smatch/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SOC development".to_string(),
            slug: "soc".to_string(),
            description: Some("Linux SOC development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/soc/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux Sophgo SoC development".to_string(),
            slug: "sophgo".to_string(),
            description: Some("Linux Sophgo SoC development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/sophgo/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux SpacemiT device drivers".to_string(),
            slug: "spacemit".to_string(),
            description: Some("Linux SpacemiT device drivers [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/spacemit/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SPARC and UltraSPARC platform development".to_string(),
            slug: "sparclinux".to_string(),
            description: Some("SPARC and UltraSPARC platform development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/sparclinux/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Storage Performance Development Kit (SPDK)".to_string(),
            slug: "spdk".to_string(),
            description: Some("Storage Performance Development Kit (SPDK) [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/spdk/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux kernel -stable discussions".to_string(),
            slug: "stable".to_string(),
            description: Some("Linux kernel -stable discussions [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/stable/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/stable/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "RealTime Linux kernel -stable discussions".to_string(),
            slug: "stable-rt".to_string(),
            description: Some("RealTime Linux kernel -stable discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/stable-rt/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SCSI target infrastructure discussions".to_string(),
            slug: "stgt".to_string(),
            description: Some("SCSI target infrastructure discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/stgt/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Archive-only list for syzbot".to_string(),
            slug: "syzbot".to_string(),
            description: Some("Archive-only list for syzbot [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/syzbot/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "SCSI target development".to_string(),
            slug: "target-devel".to_string(),
            description: Some("SCSI target development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/target-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Technical Advisory Board (TAB) public discussions".to_string(),
            slug: "tech-board-discuss".to_string(),
            description: Some("Technical Advisory Board (TAB) public discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/tech-board-discuss/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux HTE (hardware timestamp engine) subsytem".to_string(),
            slug: "timestamp".to_string(),
            description: Some("Linux HTE (hardware timestamp engine) subsytem [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/timestamp/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux maintainer tooling and workflows".to_string(),
            slug: "tools".to_string(),
            description: Some("Linux maintainer tooling and workflows [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/tools/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "TPM protocol marshaller and unmarshaller".to_string(),
            slug: "tpm-protocol".to_string(),
            description: Some("TPM protocol marshaller and unmarshaller [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/tpm-protocol/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "TPM2 (Trusted Platform Module) userspace development".to_string(),
            slug: "tpm2".to_string(),
            description: Some("TPM2 (Trusted Platform Module) userspace development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/tpm2/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "tpmdd-devel Archive on lore.kernel.org".to_string(),
            slug: "tpmdd-devel".to_string(),
            description: Some("tpmdd-devel Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/tpmdd-devel/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Trinity fuzzer tool discussions".to_string(),
            slug: "trinity".to_string(),
            description: Some("Trinity fuzzer tool discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/trinity/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "U-Boot Archive on lore.kernel.org".to_string(),
            slug: "u-boot".to_string(),
            description: Some("U-Boot Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/u-boot/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/u-boot/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "u-boot-amlogic archives".to_string(),
            slug: "u-boot-amlogic".to_string(),
            description: Some("u-boot-amlogic archives [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/u-boot-amlogic/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Ultralinux archive on lore.kernel.org".to_string(),
            slug: "ultralinux".to_string(),
            description: Some("Ultralinux archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/ultralinux/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Util-Linux package development".to_string(),
            slug: "util-linux".to_string(),
            description: Some("Util-Linux package development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/util-linux/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux 9p file system development".to_string(),
            slug: "v9fs".to_string(),
            description: Some("Linux 9p file system development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/v9fs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Discussion of the VIRTIO specification".to_string(),
            slug: "virtio-comment".to_string(),
            description: Some("Discussion of the VIRTIO specification [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/virtio-comment/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Discussion of the implementations of VIRTIO specification".to_string(),
            slug: "virtio-dev".to_string(),
            description: Some("Discussion of the implementations of VIRTIO specification [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/virtio-dev/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Development discussions about virtio-fs".to_string(),
            slug: "virtio-fs".to_string(),
            description: Some("Development discussions about virtio-fs [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/virtio-fs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Linux virtualization list".to_string(),
            slug: "virtualization".to_string(),
            description: Some("Linux virtualization list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/virtualization/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "WireGuard Archive on lore.kernel.org".to_string(),
            slug: "wireguard".to_string(),
            description: Some("WireGuard Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/wireguard/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Wireless-regdb Archive on lore.kernel.org".to_string(),
            slug: "wireless-regdb".to_string(),
            description: Some("Wireless-regdb Archive on lore.kernel.org [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/wireless-regdb/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Maintainer workflows discussions".to_string(),
            slug: "workflows".to_string(),
            description: Some("Maintainer workflows discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/workflows/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "x86-cpuid.org development mailing list".to_string(),
            slug: "x86-cpuid".to_string(),
            description: Some("x86-cpuid.org development mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/x86-cpuid/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "XDP Newbie developer discussions".to_string(),
            slug: "xdp-newbies".to_string(),
            description: Some("XDP Newbie developer discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/xdp-newbies/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "XDP2 discussion & development".to_string(),
            slug: "xdp2".to_string(),
            description: Some("XDP2 discussion & development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/xdp2/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Xen-Devel Archive on lore.kernel.org".to_string(),
            slug: "xen-devel".to_string(),
            description: Some("Xen-Devel Archive on lore.kernel.org [epoch 1]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/xen-devel/git/0.git".to_string(), order: 0 },
                RepoShard { url: "https://lore.kernel.org/xen-devel/git/1.git".to_string(), order: 1 },
            ],
        },
        MailingListSeed {
            name: "Xenomai real-time core development".to_string(),
            slug: "xenomai".to_string(),
            description: Some("Xenomai real-time core development [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/xenomai/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "XFS stable LTS mailing list".to_string(),
            slug: "xfs-stable".to_string(),
            description: Some("XFS stable LTS mailing list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/xfs-stable/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Project Discussions".to_string(),
            slug: "yocto".to_string(),
            description: Some("Yocto Project Discussions [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Project Documentation".to_string(),
            slug: "yocto-docs".to_string(),
            description: Some("Yocto Project Documentation [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-docs/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Meta Arago".to_string(),
            slug: "yocto-meta-arago".to_string(),
            description: Some("Yocto Meta Arago [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-meta-arago/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Meta Arm".to_string(),
            slug: "yocto-meta-arm".to_string(),
            description: Some("Yocto Meta Arm [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-meta-arm/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Meta Freescale".to_string(),
            slug: "yocto-meta-freescale".to_string(),
            description: Some("Yocto Meta Freescale [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-meta-freescale/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Meta TI".to_string(),
            slug: "yocto-meta-ti".to_string(),
            description: Some("Yocto Meta TI [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-meta-ti/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Meta Virtualization".to_string(),
            slug: "yocto-meta-virtualization".to_string(),
            description: Some("Yocto Meta Virtualization [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-meta-virtualization/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Patches for Yocto layers and components that do not have their own list".to_string(),
            slug: "yocto-patches".to_string(),
            description: Some("Patches for Yocto layers and components that do not have their own list [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-patches/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Project weekly status reports".to_string(),
            slug: "yocto-status".to_string(),
            description: Some("Yocto Project weekly status reports [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-status/git/0.git".to_string(), order: 0 },
            ],
        },
        MailingListSeed {
            name: "Yocto Toaster".to_string(),
            slug: "yocto-toaster".to_string(),
            description: Some("Yocto Toaster [epoch 0]".to_string()),
            repos: vec![
                RepoShard { url: "https://lore.kernel.org/yocto-toaster/git/0.git".to_string(), order: 0 },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_data_loads() {
        let lists = get_all_mailing_lists();
        assert!(lists.len() > 300, "Should have at least 300 mailing lists");
        
        // Check some well-known lists
        assert!(lists.iter().any(|l| l.slug == "lkml"));
        assert!(lists.iter().any(|l| l.slug == "bpf"));
        assert!(lists.iter().any(|l| l.slug == "netdev"));
    }

    #[test]
    fn test_multi_shard_lists() {
        let lists = get_all_mailing_lists();
        
        // lkml should have multiple shards
        let lkml = lists.iter().find(|l| l.slug == "lkml").expect("lkml should exist");
        assert!(lkml.repos.len() > 1, "lkml should have multiple repository shards");
    }
}

//! Brazilian Portuguese texts.

pub(crate) const BRAND: &str = "MACCLEANER";

// Navigation
pub(crate) const SCREEN_DASHBOARD: &str = "Painel";
pub(crate) const SCREEN_CLEANUP: &str = "Limpeza";
pub(crate) const SCREEN_DUPLICATES: &str = "Duplicados";
pub(crate) const SCREEN_PHOTOS: &str = "Fotos";
pub(crate) const SCREEN_DEVELOPER: &str = "Desenvolvedor";
pub(crate) const SCREEN_DISK_MAP: &str = "Mapa do disco";
pub(crate) const SCREEN_MONITOR: &str = "Monitor";

pub(crate) const TITLE_DASHBOARD: &str = "Central de operações";
pub(crate) const TITLE_CLEANUP: &str = "Limpeza";
pub(crate) const TITLE_DUPLICATES: &str = "Arquivos duplicados";
pub(crate) const TITLE_PHOTOS: &str = "Fotos parecidas";
pub(crate) const TITLE_DEVELOPER: &str = "Artefatos de build";
pub(crate) const TITLE_DISK_MAP: &str = "Mapa de setores";
pub(crate) const TITLE_MONITOR: &str = "Telemetria";

pub(crate) const SUBTITLE_DASHBOARD: &str =
    "Visão geral do disco, da memória e do que pode ser recuperado";
pub(crate) const SUBTITLE_CLEANUP: &str =
    "Caches, temporários e logs · prévia antes de qualquer exclusão";
pub(crate) const SUBTITLE_DUPLICATES: &str =
    "Cópias confirmadas por conteúdo · uma cópia sempre fica";
pub(crate) const SUBTITLE_PHOTOS: &str =
    "Idênticas e visualmente parecidas · a biblioteca do app Fotos não é tocada";
pub(crate) const SUBTITLE_DEVELOPER: &str =
    "Pastas recriáveis reconhecidas pelo arquivo do projeto";
pub(crate) const SUBTITLE_DISK_MAP: &str = "Cada bloco é proporcional ao espaço ocupado";
pub(crate) const SUBTITLE_MONITOR: &str = "Amostragem a cada segundo";

pub(crate) const MODULE_PENDING_TITLE: &str = "Módulo em construção";
pub(crate) const MODULE_PENDING_BODY: &str =
    "A moldura da interface está pronta. O conteúdo desta tela chega nas próximas etapas.";

// Status bar and rail
pub(crate) const FREE_SPACE_SUFFIX: &str = "livres";
pub(crate) const STARTUP_DISK: &str = "Disco de inicialização";
pub(crate) const DISK_CRITICAL: &str = "crítico";
pub(crate) const DISK_OF: &str = "de";
pub(crate) const DISK_UNKNOWN: &str = "uso do disco indisponível";

// Appearance panel
pub(crate) const APPEARANCE: &str = "Aparência";
pub(crate) const APPEARANCE_CLOSE_HINT: &str = "esc fecha";
pub(crate) const APPEARANCE_PALETTE: &str = "Paleta";
pub(crate) const APPEARANCE_MOTION: &str = "Animações";
pub(crate) const APPEARANCE_GLOW: &str = "Brilho";
pub(crate) const APPEARANCE_REPLAY_BOOT: &str = "Rever abertura";
pub(crate) const PALETTE_AGENCY: &str = "Agência";
pub(crate) const PALETTE_TERMINAL: &str = "Terminal";
pub(crate) const PALETTE_TACTICAL: &str = "Tático";
pub(crate) const MOTION_FULL: &str = "Completa";
pub(crate) const MOTION_REDUCED: &str = "Reduzida";
pub(crate) const MOTION_OFF: &str = "Desligada";

// Boot sequence: each line states something that is true when it is shown.
pub(crate) const BOOT_LINES: [(&str, &str); 4] = [
    ("> política iCloud: sem downloads", "ok"),
    ("> tema e fontes", "ok"),
    ("> disco de inicialização", "ok"),
    ("> interface", "pronta"),
];
pub(crate) const BOOT_SKIP_HINT: &str = "clique para pular";

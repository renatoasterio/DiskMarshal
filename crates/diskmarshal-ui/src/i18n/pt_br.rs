//! Brazilian Portuguese texts.

use std::net::IpAddr;
use std::time::Duration;

use diskmarshal_application::ApplicationError;
use diskmarshal_application::ports::{
    AuthenticationKind, EngineError, InspectionError, PrivilegeError, SpeedTestError,
    TrafficProbeError,
};
use diskmarshal_domain::applications::LeftoverKind;
use diskmarshal_domain::docker::ResourceKind;
use diskmarshal_domain::network::{
    Advice, ConnectionProfile, Direction, Finding, NetworkChange, NetworkReport, Optimization,
    OptimizationKind, PingResult, ProxySettings, ResponsivenessLevel, Severity, SignalQuality,
    SpeedTest, VpnProfiles, WifiLink,
};
use diskmarshal_domain::photos::SimilarityTolerance;
use diskmarshal_domain::scan::{CloudState, FileCategory};
use diskmarshal_domain::shared::ByteSize;

pub(crate) use diskmarshal_domain::shared::group_digits;

use super::Counted;

pub(crate) const BRAND: &str = "DISKMARSHAL";

// Navigation
pub(crate) const SCREEN_DASHBOARD: &str = "Painel";
pub(crate) const SCREEN_CLEANUP: &str = "Limpeza";
pub(crate) const SCREEN_APPS: &str = "Aplicativos";
pub(crate) const SCREEN_DUPLICATES: &str = "Duplicados";
pub(crate) const SCREEN_PHOTOS: &str = "Fotos";
pub(crate) const SCREEN_DEVELOPER: &str = "Desenvolvedor";
pub(crate) const SCREEN_DOCKER: &str = "Docker";
pub(crate) const SCREEN_DISK_MAP: &str = "Mapa do disco";
pub(crate) const SCREEN_MONITOR: &str = "Monitor";
pub(crate) const SCREEN_NETWORK: &str = "Rede";

pub(crate) const TITLE_DASHBOARD: &str = "Central de operações";
pub(crate) const TITLE_CLEANUP: &str = "Limpeza";
pub(crate) const TITLE_APPS: &str = "Desinstalar aplicativos";
pub(crate) const TITLE_DUPLICATES: &str = "Arquivos duplicados";
pub(crate) const TITLE_PHOTOS: &str = "Fotos parecidas";
pub(crate) const TITLE_DEVELOPER: &str = "Artefatos de build";
pub(crate) const TITLE_DOCKER: &str = "Resíduos do Docker";
pub(crate) const TITLE_DISK_MAP: &str = "Mapa de setores";
pub(crate) const TITLE_MONITOR: &str = "Telemetria";
pub(crate) const TITLE_NETWORK: &str = "Central de rede";

pub(crate) const SUBTITLE_DASHBOARD: &str =
    "Visão geral do disco, da memória e do que pode ser recuperado";
pub(crate) const SUBTITLE_CLEANUP: &str =
    "Caches, temporários e logs · prévia antes de qualquer exclusão";
pub(crate) const SUBTITLE_APPS: &str =
    "O app e tudo o que ele deixou na Biblioteca · prévia item por item antes de remover";
pub(crate) const SUBTITLE_DOCKER: &str = "Imagens sem uso, containers parados, volumes órfãos e cache de build · o que está em uso não entra na lista";
pub(crate) const SUBTITLE_DUPLICATES: &str =
    "Cópias confirmadas por conteúdo · uma cópia sempre fica";
pub(crate) const SUBTITLE_PHOTOS: &str =
    "Idênticas e visualmente parecidas · a biblioteca do app Fotos não é tocada";
pub(crate) const SUBTITLE_DEVELOPER: &str =
    "Pastas recriáveis reconhecidas pelo arquivo do projeto";
pub(crate) const SUBTITLE_DISK_MAP: &str = "Cada bloco é proporcional ao espaço ocupado";
pub(crate) const SUBTITLE_MONITOR: &str = "Amostragem a cada segundo";
pub(crate) const SUBTITLE_NETWORK: &str =
    "Leitura local dos contadores do macOS · nenhum pacote é capturado";

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

// Shared words
pub(crate) const ITEMS: Counted = Counted::new("item", "itens");
pub(crate) const CANCEL: &str = "Cancelar";
pub(crate) const OPEN_FULL_DISK_ACCESS: &str = "Abrir ajustes de Acesso Total ao Disco";
pub(crate) const FULL_DISK_ACCESS_MISSING: &str =
    "Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises.";
pub(crate) const SCAN_LOST: &str = "A varredura parou por um erro interno. Tente de novo.";

pub(crate) const TRY_AGAIN: &str = "Tentar de novo";
pub(crate) const CLOSE: &str = "Fechar";
pub(crate) const SELECTED: Counted = Counted::new("selecionado", "selecionados");
pub(crate) const SELECT_SOMETHING: &str = "Selecione ao menos um item.";
pub(crate) const EMPTY: &str = "vazio";
pub(crate) const RULES: Counted = Counted::new("regra", "regras");
pub(crate) const RISK_SAFE: &str = "Seguro";
pub(crate) const RISK_REVIEW: &str = "Revisar";

// Removal dialogs, shared by every screen that deletes
pub(crate) const CONFIRM_TITLE: &str = "Autorização requerida";
pub(crate) const REMOVING_TITLE: &str = "Removendo";
pub(crate) const REMOVING_BODY: &str = "Cada item é conferido de novo antes de ser removido.";
pub(crate) const DONE_TITLE: &str = "Concluído";
pub(crate) const FREED: &str = "liberados";
pub(crate) const ITEMS_REMOVED: Counted = Counted::new(
    "item removido · registrado no histórico",
    "itens removidos · registrados no histórico",
);
pub(crate) const SENT_TO_TRASH: &str = "enviados para a Lixeira";
pub(crate) const ITEMS_SKIPPED: Counted = Counted::new(
    "item deixado como estava: mudou, sumiu ou o macOS não permitiu",
    "itens deixados como estavam: mudaram, sumiram ou o macOS não permitiu",
);
pub(crate) const ITEMS_UNRECORDED: Counted = Counted::new(
    "remoção não pôde ser gravada no histórico",
    "remoções não puderam ser gravadas no histórico",
);

// Cleanup
pub(crate) const CLEANUP_ANALYZING: &str = "Analisando";
pub(crate) const CLEANUP_FAILED: &str = "Análise interrompida";
pub(crate) const CLEANUP_USER_GROUP: &str = "Conta do usuário";
pub(crate) const CLEANUP_USER_META: &str = "sem autenticação";
pub(crate) const CLEANUP_ADMIN_GROUP: &str = "Sistema";
pub(crate) const CLEANUP_ADMIN_UNAVAILABLE: &str = "só no app instalado · cargo xtask install";
pub(crate) const ADMIN_PROMPT: &str = "O DiskMarshal precisa de permissão de administrador para limpar as áreas de sistema que você selecionou.";
pub(crate) const CLEANUP_ACTION: &str = "Limpar selecionados";
pub(crate) const CLEANUP_AUTHORIZE: &str = "Autorizar limpeza";
pub(crate) const CLEANUP_CONFIRM_BODY: &str =
    "O conteúdo das regras selecionadas será apagado de forma permanente.";
pub(crate) const CLEANUP_CONFIRM_TRASH: &str =
    "Xcode Archives vai para a Lixeira e pode ser recuperado";
pub(crate) const CLEANUP_CONFIRM_RECHECK: &str =
    "As pastas em si ficam; só o conteúdo sai. Itens que mudarem até lá são pulados";

// Apps
pub(crate) const APPS_LISTING: &str = "Medindo aplicativos";
pub(crate) const APPS_FAILED: &str = "Listagem interrompida";
pub(crate) const APPS_CANCELLED: &str = "A listagem foi cancelada.";
pub(crate) const APPS_PLAN_FAILED: &str = "Não foi possível examinar o app";
pub(crate) const APPS_LIST_TITLE: &str = "Instalados";
pub(crate) const APPS_COUNT: Counted = Counted::new("app", "apps");
pub(crate) const APPS_SEARCH_HINT: &str = "Buscar pelo nome";
pub(crate) const APPS_NONE_MATCH: &str = "Nenhum app com esse nome.";
pub(crate) const APPS_NONE_INSTALLED: &str = "Nenhum app encontrado nas pastas de aplicativos.";
pub(crate) const APPS_VERSION: &str = "versão";
pub(crate) const APPS_OPEN: &str = "Aberto";
pub(crate) const APPS_CHOOSE_TITLE: &str = "Nenhum app escolhido";
pub(crate) const APPS_CHOOSE_BODY: &str =
    "Escolha um app na lista para ver o que a desinstalação remove.";
pub(crate) const APPS_PLANNING: &str = "Procurando o que é do app";
pub(crate) const APPS_OWNER_USER: &str = "instalado por você";
pub(crate) const APPS_OWNER_SYSTEM: &str = "instalado pela App Store ou por um instalador";
pub(crate) const APPS_OPEN_NOW: &str = "aberto agora";
pub(crate) const APPS_IDENTIFIER: &str = "Identificador";
pub(crate) const APPS_VERSION_LABEL: &str = "Versão";
pub(crate) const APPS_FOUND: &str = "Encontrado";
pub(crate) const APPS_OTHER_COPY: &str =
    "Há outra cópia deste app instalada. Os dados são das duas, então só o app em si é oferecido.";
pub(crate) const APPS_USER_META: &str = "vai para a Lixeira · pode ser recuperado";
pub(crate) const APPS_ACTION: &str = "Desinstalar";
pub(crate) const APPS_CHOOSE_FIRST: &str = "Escolha um app na lista.";
pub(crate) const APPS_CONFIRM_TRASH: &str =
    "Os itens da sua conta vão para a Lixeira e podem ser recuperados de lá";
pub(crate) const APPS_CONFIRM_REVIEW: &str =
    "Itens com o selo Revisar podem ser de outro app: confira antes";
pub(crate) const APPS_ADMIN_PROMPT: &str = "O DiskMarshal precisa de permissão de administrador para remover os itens de sistema do app que você está desinstalando.";

/// What the confirmation of an uninstall says first.
pub(crate) fn apps_confirm_body(name: &str) -> String {
    format!("Os itens marcados de {name} saem deste Mac.")
}

/// Why an app that is open cannot be uninstalled.
pub(crate) fn apps_running(name: &str) -> String {
    format!("{name} está aberto. Feche o app para poder desinstalar.")
}

/// Why an uninstall that was authorized removed nothing.
pub(crate) fn apps_opened_meanwhile(name: &str) -> String {
    format!("Nada foi removido: {name} foi aberto. Feche o app e tente de novo.")
}

/// Name of a kind of item an uninstall removes.
pub(crate) const fn leftover_kind(kind: LeftoverKind) -> &'static str {
    match kind {
        LeftoverKind::Application => "Aplicativo",
        LeftoverKind::Support => "Dados do app",
        LeftoverKind::Caches => "Caches",
        LeftoverKind::Preferences => "Preferências",
        LeftoverKind::Container => "Contêiner",
        LeftoverKind::SavedState => "Estado salvo",
        LeftoverKind::WebData => "Dados web",
        LeftoverKind::Logs => "Logs",
        LeftoverKind::LaunchItem => "Serviço",
        LeftoverKind::Helper => "Auxiliar",
    }
}

// Docker
pub(crate) const DOCKER_INSPECTING: &str = "Perguntando ao Docker";
pub(crate) const DOCKER_INSPECTING_BODY: &str =
    "O Docker mede cada imagem e cada volume para responder. Com muitos, leva alguns segundos.";
pub(crate) const DOCKER_MISSING_TITLE: &str = "Docker não encontrado";
pub(crate) const DOCKER_MISSING_BODY: &str =
    "Nenhuma instalação do Docker foi encontrada neste Mac, então não há o que limpar aqui.";
pub(crate) const DOCKER_MISSING_NOTE: &str =
    "Procurado em /usr/local/bin, /opt/homebrew/bin, Docker Desktop, OrbStack e Rancher Desktop.";
pub(crate) const DOCKER_STOPPED_TITLE: &str = "Docker parado";
pub(crate) const DOCKER_STOPPED_META: &str = "instalado · sem resposta";
pub(crate) const DOCKER_STOPPED_BODY: &str = "O Docker está instalado, mas não está em execução. Imagens, volumes e cache ficam dentro dele: só dá para medir e limpar com ele aberto.";
pub(crate) const DOCKER_STOPPED_NOTE: &str =
    "Abra o Docker, espere ele terminar de iniciar e verifique de novo.";
pub(crate) const DOCKER_FAILED_TITLE: &str = "O Docker não respondeu";
pub(crate) const DOCKER_CHECK_AGAIN: &str = "Verificar de novo";
pub(crate) const DOCKER_MEASURE_AGAIN: &str = "Medir de novo";
pub(crate) const DOCKER_RUNNING: &str = "em execução";
pub(crate) const DOCKER_UNUSED: &str = "sem uso";
pub(crate) const DOCKER_IN_USE: &str = "em uso";
pub(crate) const DOCKER_RECORDS: Counted = Counted::new("registro", "registros");
pub(crate) const DOCKER_LIST_TITLE: &str = "Sem uso";
pub(crate) const DOCKER_LIST_META: &str = "o que está em uso não entra na lista";
pub(crate) const DOCKER_NOTHING_UNUSED: &str =
    "Nada sobrando: tudo o que o Docker guarda está em uso.";
pub(crate) const DOCKER_NOTHING_OF_KIND: &str = "Nada sem uso deste tipo.";
pub(crate) const DOCKER_UNNAMED: &str = "sem nome";
pub(crate) const DOCKER_DETAIL_CACHE: &str = "refeito no próximo build";
pub(crate) const DOCKER_DETAIL_LEFTOVER: &str = "sobra de build";
pub(crate) const DOCKER_DETAIL_UNUSED: &str = "nenhum container usa";
pub(crate) const DOCKER_DETAIL_STOPPED: &str = "parado";
pub(crate) const DOCKER_DETAIL_VOLUME: &str = "pode guardar um banco de dados";
pub(crate) const DOCKER_UP_TO: &str = "até";
pub(crate) const DOCKER_CONFIRM_BODY: &str =
    "Os itens marcados são apagados pelo próprio Docker, de forma permanente.";
pub(crate) const DOCKER_CONFIRM_REBUILD: &str = "Imagens e cache voltam com um novo pull ou build";
pub(crate) const DOCKER_CONFIRM_DATA: &str = "O conteúdo de volumes e containers não tem volta";
pub(crate) const DOCKER_CONFIRM_IN_USE: &str =
    "O que estiver em uso na hora é recusado pelo Docker e fica como está";
pub(crate) const DOCKER_SKIPPED: Counted = Counted::new(
    "item recusado pelo Docker: estava em uso ou ele não respondeu",
    "itens recusados pelo Docker: estavam em uso ou ele não respondeu",
);

/// Name of one Docker resource of a kind.
pub(crate) const fn docker_kind(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::BuildCache => "Cache de build",
        ResourceKind::Image => "Imagem",
        ResourceKind::Container => "Container",
        ResourceKind::Volume => "Volume",
    }
}

/// Name of everything Docker holds of a kind, and of what is counted.
pub(crate) const fn docker_kind_plural(kind: ResourceKind) -> (&'static str, Counted) {
    match kind {
        ResourceKind::BuildCache => ("Cache de build", DOCKER_RECORDS),
        ResourceKind::Image => ("Imagens", Counted::new("imagem", "imagens")),
        ResourceKind::Container => ("Containers", Counted::new("container", "containers")),
        ResourceKind::Volume => ("Volumes", Counted::new("volume", "volumes")),
    }
}

/// Says why Docker could not be asked.
pub(crate) fn engine_error(error: &EngineError) -> String {
    match error {
        EngineError::NotInstalled => DOCKER_MISSING_BODY.to_owned(),
        EngineError::NotRunning => DOCKER_STOPPED_BODY.to_owned(),
        EngineError::TimedOut => "O Docker demorou demais para responder.".to_owned(),
        EngineError::Failed(reason) => format!("O Docker respondeu com um erro: {reason}"),
    }
}

pub(crate) const ALL: &str = "Todos";
pub(crate) const CHOOSE_FOLDER: &str = "Escolher pasta";

// Developer
pub(crate) const DEV_INTRO_TITLE: &str = "Projetos de código";
pub(crate) const DEV_INTRO_BODY: &str = "Escolha onde ficam os seus projetos. O DiskMarshal procura pastas que as ferramentas recriam sozinhas, como node_modules, target e bin/obj.";
pub(crate) const DEV_INTRO_NOTE: &str = "Uma pasta só é oferecida quando o arquivo do projeto está ao lado dela e nada ali é versionado no git.";
pub(crate) const DEV_SEARCHING: &str = "Procurando projetos";
pub(crate) const DEV_LIST_TITLE: &str = "Pastas recriáveis";
pub(crate) const DEV_NOTHING_FOUND: &str = "Nenhuma pasta de build encontrada aqui.";
pub(crate) const DEV_SEARCH_AGAIN: &str = "Procurar de novo";
pub(crate) const DEV_TOUCHED: &str = "alterado";
pub(crate) const DEV_ACTION: &str = "Excluir selecionados";
pub(crate) const DEV_AUTHORIZE: &str = "Autorizar exclusão";
pub(crate) const DEV_FOLDERS: Counted = Counted::new("pasta", "pastas");
pub(crate) const DEV_CONFIRM_BODY: &str =
    "As pastas selecionadas serão apagadas de forma permanente.";
pub(crate) const DEV_CONFIRM_SOURCE: &str = "Nenhum código-fonte é removido";
pub(crate) const DEV_CONFIRM_RECREATE: &str =
    "Elas voltam na próxima instalação de dependências ou compilação";

pub(crate) const MOVE_TO_TRASH: &str = "Mover para a Lixeira";
pub(crate) const TRASH_FREES_LATER: &str = "O espaço é liberado quando você esvaziar a Lixeira";
pub(crate) const IN_TRASH: &str = "na Lixeira";

// Duplicates
pub(crate) const DUP_INTRO_TITLE: &str = "Arquivos duplicados";
pub(crate) const DUP_INTRO_BODY: &str = "Escolha onde procurar. Dois arquivos só contam como duplicados quando o conteúdo inteiro é idêntico.";
pub(crate) const DUP_INTRO_NOTE: &str =
    "Arquivos que estão só no iCloud ficam de fora: compará-los exigiria baixá-los.";
pub(crate) const DUP_MINIMUM: &str = "Tamanho mínimo";
pub(crate) const DUP_SEARCHING: &str = "Procurando duplicados";
pub(crate) const DUP_SEARCHING_META: &str = "medindo e comparando conteúdo";
pub(crate) const DUP_NONE_TITLE: &str = "Nenhum duplicado";
pub(crate) const DUP_NONE_BODY: &str =
    "Não há arquivos idênticos nesta pasta com o tamanho mínimo escolhido.";
pub(crate) const DUP_KEEP: &str = "Manter";
pub(crate) const DUP_KEEP_ONE: &str = "Ao menos uma cópia de cada arquivo precisa ficar.";
pub(crate) const DUP_COPIES: Counted = Counted::new("cópia", "cópias");
pub(crate) const DUP_EACH: &str = "cada";
pub(crate) const DUP_FILES: Counted = Counted::new("arquivo", "arquivos");
pub(crate) const DUP_MORE_PREFIX: &str = "Mais";
pub(crate) const DUP_MORE_SUFFIX: Counted = Counted::new(
    "grupo menor não é mostrado. Remova os maiores e procure de novo.",
    "grupos menores não são mostrados. Remova os maiores e procure de novo.",
);
pub(crate) const DUP_CONFIRM_BODY: &str =
    "As cópias marcadas vão para a Lixeira e podem ser recuperadas.";
pub(crate) const DUP_CONFIRM_KEEP: &str = "Uma cópia de cada arquivo é mantida";

// Photos
pub(crate) const PHOTOS_INTRO_BODY: &str = "Escolha onde procurar. São comparadas só as imagens soltas em pastas; a biblioteca do app Fotos nunca é lida nem alterada.";
pub(crate) const PHOTOS_INTRO_NOTE: &str = "Formatos: JPEG, PNG, HEIC, TIFF, WebP e RAW de câmera. Imagens com menos de 50 KB e arquivos que estão só no iCloud ficam de fora.";
pub(crate) const PHOTOS_SEARCH_PICTURES: &str = "Procurar em Imagens";
pub(crate) const PHOTOS_TOLERANCE: &str = "Semelhança";
pub(crate) const PHOTOS_SEARCHING: &str = "Procurando fotos parecidas";
pub(crate) const PHOTOS_SEARCHING_META: &str = "medindo a pasta, depois lendo cada foto";
pub(crate) const PHOTOS_NONE_TITLE: &str = "Nenhuma foto parecida";
pub(crate) const PHOTOS_COMPARED: Counted = Counted::new("foto comparada.", "fotos comparadas.");
pub(crate) const PHOTOS_NONE_BODY: &str =
    "Nenhuma delas se parece com outra neste nível de semelhança.";
pub(crate) const PHOTOS_GROUP: Counted = Counted::new("foto parecida", "fotos parecidas");
pub(crate) const PHOTOS_FREES: &str = "libera";
pub(crate) const PHOTOS_REFERENCE: &str = "REFERÊNCIA";
pub(crate) const PHOTOS_MATCH: &str = "MATCH";
pub(crate) const PHOTOS_REMOVE: &str = "Remover";
pub(crate) const PHOTOS_NO_PICTURE: &str = "sem prévia";
pub(crate) const PHOTOS_KEEP_ONE: &str = "Ao menos uma foto de cada grupo precisa ficar.";
pub(crate) const PHOTOS_MARKED: Counted = Counted::new("foto marcada", "fotos marcadas");
pub(crate) const PHOTOS_COUNT: Counted = Counted::new("foto", "fotos");
pub(crate) const PHOTOS_MORE_SUFFIX: Counted = Counted::new(
    "grupo menor não é mostrado. Remova estes e procure de novo.",
    "grupos menores não são mostrados. Remova estes e procure de novo.",
);
pub(crate) const PHOTOS_CONFIRM_BODY: &str =
    "As fotos marcadas vão para a Lixeira e podem ser recuperadas de lá.";
pub(crate) const PHOTOS_CONFIRM_KEEP: &str = "Ao menos uma foto de cada grupo é mantida";

/// The name of a level of similarity, for the filter.
pub(crate) const fn similarity_tolerance(tolerance: SimilarityTolerance) -> &'static str {
    match tolerance {
        SimilarityTolerance::Strict => "Quase idênticas",
        SimilarityTolerance::Balanced => "Parecidas",
        SimilarityTolerance::Loose => "Ampla",
    }
}

// Monitor
pub(crate) const MONITOR_WAITING_TITLE: &str = "Aguardando leituras";
pub(crate) const MONITOR_WAITING_BODY: &str = "A primeira leitura chega em cerca de um segundo. Se nada aparecer, o macOS não informou os dados do sistema.";
pub(crate) const MONITOR_CPU: &str = "CPU";
pub(crate) const MONITOR_PER_CORE: &str = "Núcleos";
pub(crate) const MONITOR_CORES: Counted = Counted::new("núcleo", "núcleos");
pub(crate) const MONITOR_MEMORY: &str = "Memória";
pub(crate) const MONITOR_SWAP: &str = "Swap em uso";
pub(crate) const MONITOR_FREE: &str = "Livre";
pub(crate) const MONITOR_IN_USE: &str = "em uso";
pub(crate) const MONITOR_VOLUME: &str = "Volume";
pub(crate) const MONITOR_STARTUP_VOLUME: &str = "disco de inicialização";
pub(crate) const MONITOR_DISK: &str = "Disco";
pub(crate) const MONITOR_TEMPERATURE: &str = "Temperatura";
pub(crate) const MONITOR_NO_SENSORS: &str = "Este Mac não informou nenhum sensor de temperatura.";
pub(crate) const MONITOR_PROCESSES: &str = "Processos";
pub(crate) const MONITOR_PROCESSES_META: &str = "por uso de CPU · 100% é um núcleo inteiro";

// Dashboard
pub(crate) const DASH_DISK: &str = "Disco";
pub(crate) const DASH_MEMORY: &str = "Memória";
pub(crate) const DASH_TEMPERATURE: &str = "Temperatura";
pub(crate) const DASH_CPU_USE: &str = "uso total";
pub(crate) const DASH_RECOVERABLE: &str = "Espaço recuperável";
pub(crate) const DASH_ROW_CLEANUP: &str = "Caches e temporários";
pub(crate) const DASH_ROW_DEVELOPER: &str = "Projetos de código";
pub(crate) const DASH_ROW_DOCKER: &str = "Resíduos do Docker";
pub(crate) const DASH_ROW_DUPLICATES: &str = "Arquivos duplicados";
pub(crate) const DASH_ROW_PHOTOS: &str = "Fotos parecidas";
pub(crate) const DASH_NOT_ANALYZED: &str = "não analisado";
pub(crate) const DASH_HISTORY: &str = "Últimas remoções";
pub(crate) const DASH_HISTORY_META: &str = "do histórico";
pub(crate) const DASH_HISTORY_EMPTY: &str =
    "Nada foi removido ainda. Cada remoção fica registrada aqui.";
pub(crate) const DASH_DELETED: &str = "apagado";
pub(crate) const DASH_TRASHED: &str = "na Lixeira";
pub(crate) const DASH_SYSTEM: &str = "Sistema";
pub(crate) const DASH_NO_DOWNLOADS: &str = "sem downloads";
pub(crate) const DASH_ANALYZE: &str = "Analisar limpeza";

// Disk map
pub(crate) const MAP_INTRO_TITLE: &str = "Varredura";
pub(crate) const MAP_INTRO_BODY: &str = "Escolha o que medir. A varredura só lê nomes e tamanhos: nada é alterado nem baixado do iCloud.";
pub(crate) const MAP_INTRO_NOTE: &str =
    "O macOS pode pedir permissão para ler Documentos, Mesa e Downloads.";
pub(crate) const MAP_SCAN_HOME: &str = "Pasta pessoal";
pub(crate) const MAP_SCAN_DISK: &str = "Disco inteiro";
pub(crate) const MAP_SCANNING_TITLE: &str = "Varrendo";
pub(crate) const MAP_SCANNING_META: &str = "iCloud: sem downloads";
pub(crate) const MAP_RESCAN: &str = "Varrer novamente";
pub(crate) const MAP_COLOR_BY: &str = "Cor por";
pub(crate) const MAP_COLOR_TYPE: &str = "Tipo";
pub(crate) const MAP_COLOR_CLOUD: &str = "iCloud";
pub(crate) const MAP_EMPTY_FOLDER: &str = "Esta pasta está vazia.";
pub(crate) const MAP_OTHERS: &str = "outros";
pub(crate) const MAP_CARD_SIZE: &str = "Tamanho";
pub(crate) const MAP_CARD_ITEMS: &str = "Itens";
pub(crate) const MAP_CARD_CLOUD: &str = "iCloud";
pub(crate) const MAP_CARD_TYPE: &str = "Tipo";
pub(crate) const MAP_CARD_ENTER: &str = "clique: ampliar · direito: Finder";
pub(crate) const MAP_CARD_REVEAL: &str = "clique direito: mostrar no Finder";
pub(crate) const MAP_BLOCKED_FOLDERS: Counted =
    Counted::new("pasta sem acesso", "pastas sem acesso");

/// Name of a cloud state.
pub(crate) const fn cloud_state(state: CloudState) -> &'static str {
    match state {
        CloudState::Local => "Somente local",
        CloudState::Synced => "iCloud, baixado",
        CloudState::CloudOnly => "Somente na nuvem",
    }
}

/// Name of a content category.
pub(crate) const fn category(category: FileCategory) -> &'static str {
    match category {
        FileCategory::System => "Sistema e apps",
        FileCategory::Code => "Código",
        FileCategory::Photo => "Fotos",
        FileCategory::Media => "Vídeo e áudio",
        FileCategory::Document => "Documentos",
        FileCategory::Archive => "Compactados",
        FileCategory::Other => "Outros",
    }
}

/// Explains a failed operation to the user.
pub(crate) fn application_error(error: &ApplicationError) -> String {
    match error {
        ApplicationError::RootNotFound(path) => {
            format!("Pasta não encontrada: {}", path.display())
        }
        ApplicationError::RootNotDirectory(path) => format!("Não é uma pasta: {}", path.display()),
        ApplicationError::Cancelled => "Operação cancelada.".to_owned(),
        ApplicationError::TreeCapacityExceeded => {
            "Há itens demais para uma única varredura. Escolha uma pasta menor.".to_owned()
        }
        ApplicationError::InvalidCatalog(reason) => {
            format!("O catálogo de regras do app está com defeito: {reason}")
        }
        ApplicationError::RuleNotAllowed(rule) => {
            format!("A regra {rule} não pode ser executada aqui.")
        }
        ApplicationError::AppNotFound(identifier) => {
            format!("Nenhum app instalado tem o identificador {identifier}, ou há mais de um.")
        }
        ApplicationError::AppRunning(name) => {
            format!("{name} está aberto. Feche o app e tente de novo.")
        }
        ApplicationError::ItemNotAllowed(path) => {
            format!("Este item não pode ser removido aqui: {}", path.display())
        }
        ApplicationError::NetworkChangeNotAllowed(reason) => {
            format!("Esta mudança de rede não pode ser feita aqui: {reason}")
        }
        ApplicationError::NetworkChangeFailed(reason) => {
            format!("O macOS recusou a mudança de rede: {reason}")
        }
        ApplicationError::Io { path, source } => {
            format!("O macOS recusou o acesso a {}: {source}", path.display())
        }
    }
}

/// What a group of administrator actions says about how macOS will ask.
pub(crate) const fn admin_meta(authentication: AuthenticationKind) -> &'static str {
    match authentication {
        AuthenticationKind::TouchId => "pede a sua digital · senha se ela falhar",
        AuthenticationKind::Password => "pede a senha do macOS",
    }
}

/// The line of the cleanup confirmation about the system areas.
pub(crate) const fn cleanup_confirm_admin(authentication: AuthenticationKind) -> &'static str {
    match authentication {
        AuthenticationKind::TouchId => {
            "As áreas de sistema pedem a sua digital (Touch ID), no diálogo do próprio macOS"
        }
        AuthenticationKind::Password => {
            "As áreas de sistema pedem a senha de administrador, no diálogo do próprio macOS"
        }
    }
}

/// What the system group of an uninstall says about how macOS will ask.
pub(crate) const fn apps_admin_meta(authentication: AuthenticationKind) -> &'static str {
    match authentication {
        AuthenticationKind::TouchId => "pede a sua digital · apagado de vez",
        AuthenticationKind::Password => "pede a senha · apagado de vez",
    }
}

/// The line of the uninstall confirmation about the system items.
pub(crate) const fn apps_confirm_admin(authentication: AuthenticationKind) -> &'static str {
    match authentication {
        AuthenticationKind::TouchId => {
            "Os itens de sistema pedem a sua digital (Touch ID) e são apagados de vez"
        }
        AuthenticationKind::Password => {
            "Os itens de sistema pedem a senha de administrador e são apagados de vez"
        }
    }
}

/// Says why the system areas were not cleaned.
pub(crate) fn privilege_error(error: &PrivilegeError) -> String {
    administrator_failure("As áreas de sistema não foram limpas", error)
}

/// Says why the system items of an app were not removed.
pub(crate) fn uninstall_privilege_error(error: &PrivilegeError) -> String {
    administrator_failure("Os itens de sistema não foram removidos", error)
}

/// Says, after `what` did not happen, why the administrator part failed.
fn administrator_failure(what: &str, error: &PrivilegeError) -> String {
    match error {
        PrivilegeError::Cancelled => format!("{what}: a autenticação foi cancelada."),
        PrivilegeError::Unavailable(_) => {
            format!("{what}: esse recurso só existe no app instalado.")
        }
        PrivilegeError::Failed(reason) => format!("{what}: {reason}"),
    }
}

/// Says how long ago something happened: "hoje", "há 3 dias", "há 4 meses".
pub(crate) fn age(seconds: i64) -> String {
    const DAY: u64 = 86_400;
    const DAYS: Counted = Counted::new("dia", "dias");
    const MONTHS: Counted = Counted::new("mês", "meses");
    const YEARS: Counted = Counted::new("ano", "anos");
    let days = seconds.max(0).unsigned_abs() / DAY;
    let counted = match days {
        0 => return "hoje".to_owned(),
        1..=29 => DAYS.of(days),
        30..=364 => MONTHS.of(days / 30),
        _ => YEARS.of(days / 365),
    };
    format!("há {counted}")
}

// Network
pub(crate) const NETWORK_TAB_TRAFFIC: &str = "Tráfego";
pub(crate) const NETWORK_TAB_ANALYSIS: &str = "Análise";
pub(crate) const NETWORK_TAB_TUNING: &str = "Otimização";
pub(crate) const NETWORK_NOW: &str = "Tráfego agora";
pub(crate) const NETWORK_SESSION: &str = "Desde que a tela abriu";
pub(crate) const NETWORK_RECEIVED: &str = "Recebido";
pub(crate) const NETWORK_SENT: &str = "Enviado";
pub(crate) const NETWORK_PEAK: &str = "Pico de download";
pub(crate) const NETWORK_CONNECTIONS_WITH_DATA: &str = "Conexões com dados";
pub(crate) const NETWORK_PROGRAMS_ON: &str = "Programas na rede";
pub(crate) const NETWORK_RESENT: &str = "Reenviado";
pub(crate) const NETWORK_PROGRAMS: &str = "Programas";
pub(crate) const NETWORK_PROGRAMS_META: &str = "quem está usando a rede";
pub(crate) const NETWORK_COLUMN_PROGRAM: &str = "Programa";
pub(crate) const NETWORK_COLUMN_RECEIVING: &str = "Recebendo";
pub(crate) const NETWORK_COLUMN_SENDING: &str = "Enviando";
pub(crate) const NETWORK_CONNECTIONS: &str = "Conexões";
pub(crate) const NETWORK_OPEN: Counted = Counted::new("aberta", "abertas");
pub(crate) const NETWORK_COLUMN_DESTINATION: &str = "Destino";
pub(crate) const NETWORK_COLUMN_ANSWER: &str = "Resposta";
pub(crate) const NETWORK_WAITING_TITLE: &str = "Aguardando leituras";
pub(crate) const NETWORK_WAITING_BODY: &str = "A primeira leitura só marca o ponto de partida dos contadores. As velocidades aparecem na segunda, em cerca de dois segundos.";
pub(crate) const NETWORK_UNREADABLE_TITLE: &str = "Tráfego indisponível";
pub(crate) const NETWORK_NO_TRAFFIC: &str = "Nenhum programa moveu dados pela rede.";
pub(crate) const NETWORK_NO_CONNECTIONS: &str = "Escolha um programa para ver as conexões dele.";
pub(crate) const NETWORK_NOT_MEASURED: &str = "—";
pub(crate) const NETWORK_TASK_LOST: &str = "A operação parou por um erro interno. Tente de novo.";
pub(crate) const NETWORK_CANCELLED: &str = "A autenticação foi cancelada. Nada foi alterado.";

pub(crate) const NETWORK_DOWNLOAD: &str = "Download";
pub(crate) const NETWORK_UPLOAD: &str = "Upload";
pub(crate) const NETWORK_LATENCY: &str = "Latência";
pub(crate) const NETWORK_RESPONSIVENESS: &str = "Resposta sob carga";
pub(crate) const NETWORK_UNIT_MEGABITS: &str = "Mb/s";
pub(crate) const NETWORK_UNIT_MILLIS: &str = "ms";
pub(crate) const NETWORK_UNIT_RPM: &str = "RPM";
pub(crate) const NETWORK_SPEED_NOTE: &str = "A medição usa o programa de teste do macOS e servidores da Apple, por cerca de 20 segundos. Numa conexão de 500 Mb/s ela transfere mais de 1 GB.";
pub(crate) const NETWORK_SPEED_RUNNING: &str =
    "Medindo download e upload ao mesmo tempo. A rede fica ocupada por cerca de 20 segundos.";
pub(crate) const NETWORK_MEASURE_SPEED: &str = "Medir velocidade";
pub(crate) const NETWORK_MEASURE_AGAIN: &str = "Medir de novo";
pub(crate) const NETWORK_ANALYZE: &str = "Analisar conexão";
pub(crate) const NETWORK_ANALYZE_AGAIN: &str = "Analisar de novo";
pub(crate) const NETWORK_ANALYZE_TITLE: &str = "Análise da conexão";
pub(crate) const NETWORK_ANALYZE_BODY: &str = "A análise lê os ajustes de rede deste Mac e o sinal do Wi-Fi, e mede quanto o roteador, a internet e os servidores de nomes demoram para responder. Para isso ela envia alguns pings e consultas: poucos kilobytes.";
pub(crate) const NETWORK_ANALYZING: &str = "Analisando a conexão. Leva cerca de dez segundos.";
pub(crate) const NETWORK_WIFI: &str = "Wi-Fi";
pub(crate) const NETWORK_SIGNAL: &str = "Sinal";
pub(crate) const NETWORK_NOISE: &str = "Ruído";
pub(crate) const NETWORK_RATE: &str = "Taxa";
pub(crate) const NETWORK_NO_WIFI: &str = "Este Mac não está ligado à rede por Wi-Fi.";
pub(crate) const NETWORK_PATH: &str = "Caminho até a internet";
pub(crate) const NETWORK_ROUTER: &str = "Roteador";
pub(crate) const NETWORK_INTERNET: &str = "Internet";
pub(crate) const NETWORK_DNS_IN_USE: &str = "DNS em uso";
pub(crate) const NETWORK_PROXY: &str = "Proxy";
pub(crate) const NETWORK_VPN: &str = "VPN";
pub(crate) const NETWORK_DIAGNOSIS: &str = "Diagnóstico";
pub(crate) const NETWORK_ATTENTION_POINTS: Counted =
    Counted::new("ponto de atenção", "pontos de atenção");

pub(crate) const NETWORK_TUNING_INTRO: &str = "Ações escolhidas a partir da análise deste Mac. Cada uma diz o que muda, só é aplicada depois da sua confirmação e pode ser desfeita aqui.";
pub(crate) const NETWORK_TUNING_NEEDS_ANALYSIS: &str = "As ações são escolhidas a partir da análise da conexão. Analise a conexão para ver o que pode ser melhorado neste Mac.";
pub(crate) const NETWORK_ACTIONS: &str = "Ações";
pub(crate) const NETWORK_APPLIED: &str = "aplicada";
pub(crate) const NETWORK_ADVICE: &str = "Orientações";
pub(crate) const NETWORK_ADVICE_META: &str = "o app não muda isto por você";
pub(crate) const NETWORK_NO_ADVICE: &str = "Nada a sugerir para esta conexão.";
pub(crate) const NETWORK_NO_TCP_TWEAKS: &str = "O DiskMarshal não altera parâmetros TCP do sistema. No macOS atual esses \"aceleradores\" não trazem ganho medido e podem piorar a conexão.";
pub(crate) const NETWORK_SELECTED_ACTIONS: Counted =
    Counted::new("ação selecionada", "ações selecionadas");
pub(crate) const NETWORK_APPLY: &str = "Aplicar selecionadas";
pub(crate) const NETWORK_UNDO: &str = "Desfazer alterações";
pub(crate) const NETWORK_NOTHING_SELECTED: &str = "Marque ao menos uma ação.";
pub(crate) const NETWORK_TUNER_UNAVAILABLE: &str =
    "Mudar ajustes de rede só funciona no app instalado.";
pub(crate) const NETWORK_CONFIRM_TITLE: &str = "Autorização requerida";
pub(crate) const NETWORK_CONFIRM_BODY: &str =
    "As ações marcadas mudam ajustes de rede deste Mac. Nenhum arquivo é apagado.";
pub(crate) const NETWORK_CONFIRM_APPLY: &str = "Aplicar";
pub(crate) const NETWORK_UNDO_BODY: &str = "Cada ajuste volta ao que era antes de ser aplicado:";
pub(crate) const NETWORK_UNDO_APPLY: &str = "Desfazer";
pub(crate) const NETWORK_APPLYING_TITLE: &str = "Aplicando";
pub(crate) const NETWORK_APPLIED_TITLE: &str = "Aplicado";
pub(crate) const NETWORK_APPLIED_BODY: &str = "O que mudou neste Mac:";
pub(crate) const NETWORK_APPLIED_NOTE: &str =
    "\"Desfazer alterações\" devolve cada ajuste ao que era.";
pub(crate) const NETWORK_UNDONE_TITLE: &str = "Desfeito";
pub(crate) const NETWORK_UNDONE_BODY: &str = "Os ajustes voltaram ao que eram:";
pub(crate) const NETWORK_FAILED_TITLE: &str = "Não aplicado";
pub(crate) const NETWORK_ADMIN_PROMPT: &str = "O DiskMarshal precisa de permissão de administrador para mudar os ajustes de rede que você selecionou.";

/// Writes a number with `places` decimal places and the pt-BR decimal comma.
pub(crate) fn decimal(value: f64, places: usize) -> String {
    format!("{value:.places$}").replace('.', ",")
}

/// Writes an answer time in whole milliseconds: "14 ms".
pub(crate) fn millis(time: Duration) -> String {
    match time.as_millis() {
        0 => "menos de 1 ms".to_owned(),
        millis => format!("{millis} ms"),
    }
}

/// The note under the traffic plot: what it spans and what its top stands for.
pub(crate) fn network_scale(top_megabits: f32) -> String {
    format!("últimos 2 minutos · topo da escala em {top_megabits:.0} Mb/s")
}

/// The title of the panel that lists the connections of `program`.
pub(crate) fn network_connections_title(program: &str) -> String {
    format!("{NETWORK_CONNECTIONS} · {program}")
}

/// Says how much of what a connection sent had to be sent again.
pub(crate) fn network_resent(amount: ByteSize) -> String {
    format!("reenvio {amount}")
}

/// Says where the traffic leaves this Mac: "Wi-Fi · en0 · 192.168.0.24".
pub(crate) fn network_interface(profile: &ConnectionProfile) -> String {
    [
        Some(profile.service.clone()).filter(|service| !service.is_empty()),
        Some(profile.interface.clone()).filter(|interface| !interface.is_empty()),
        profile.local_address.map(|address| address.to_string()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Says what a finished speed test cost.
pub(crate) fn network_speed_done(test: &SpeedTest) -> String {
    format!(
        "Medido agora · {} transferidos em {} s",
        test.transferred,
        test.duration.as_secs()
    )
}

/// The step a responsiveness figure falls in.
pub(crate) const fn network_responsiveness_level(level: ResponsivenessLevel) -> &'static str {
    match level {
        ResponsivenessLevel::Low => "baixa",
        ResponsivenessLevel::Medium => "média",
        ResponsivenessLevel::High => "alta",
    }
}

/// The line under the wireless figures: channel, band, width and neighbors.
pub(crate) fn network_wifi_channel(link: &WifiLink) -> String {
    const NEIGHBORS: Counted = Counted::new(
        "rede vizinha no mesmo canal",
        "redes vizinhas no mesmo canal",
    );
    [
        Some(format!("Canal {}", link.channel)),
        link.band.map(|band| band.label().to_owned()),
        link.width_mhz.map(|width| format!("{width} MHz")),
        Some(NEIGHBORS.of(link.neighbors_on_channel)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

/// The note beside the wireless panel title: "802.11ax · Wi-Fi 6".
pub(crate) fn network_wifi_standard(link: &WifiLink) -> String {
    match link.generation() {
        Some(generation) => format!("{} · {generation}", link.standard),
        None => link.standard.clone(),
    }
}

/// Says what pinging the router found. Many routers ignore pings on purpose.
pub(crate) fn network_router(address: Option<IpAddr>, ping: Option<&PingResult>) -> String {
    let Some(address) = address else {
        return NETWORK_NOT_MEASURED.to_owned();
    };
    match ping.and_then(|ping| ping.average) {
        Some(average) => format!("{address} · {}", millis(average)),
        None => format!("{address} · não responde a ping"),
    }
}

/// Says what pinging the internet found.
pub(crate) fn network_internet(ping: Option<&PingResult>) -> String {
    match ping {
        Some(ping) if ping.sent == 0 => "não foi possível medir".to_owned(),
        Some(ping) => match ping.average {
            Some(average) => format!(
                "{} · {:.0}% de perda",
                millis(average),
                ping.loss_ratio() * 100.0
            ),
            None => "sem resposta".to_owned(),
        },
        None => NETWORK_NOT_MEASURED.to_owned(),
    }
}

/// Says which name server is in use and how long it takes to answer.
pub(crate) fn network_dns_in_use(report: &NetworkReport) -> String {
    let Some(server) = report.profile.dns_servers.first() else {
        return "nenhum".to_owned();
    };
    match report.current_dns_time() {
        Some(time) => format!("{server} · {}", millis(time)),
        None => format!("{server} · sem resposta"),
    }
}

/// Says how connections look for a proxy.
pub(crate) const fn network_proxy(proxy: ProxySettings) -> &'static str {
    if proxy.is_auto_discovery_on {
        "descoberta automática ligada"
    } else if proxy.has_configured_proxy {
        "configurado"
    } else {
        "nenhum"
    }
}

/// Says how many VPN profiles exist and how many are connected.
pub(crate) fn network_vpn(vpn: VpnProfiles) -> String {
    const PROFILES: Counted = Counted::new("perfil", "perfis");
    const ACTIVE: Counted = Counted::new("ativo", "ativos");
    match (vpn.installed, vpn.connected) {
        (0, _) => "nenhum perfil".to_owned(),
        (installed, 0) => format!("{} · nenhum ativo", PROFILES.of(installed)),
        (installed, connected) => {
            format!("{} · {}", PROFILES.of(installed), ACTIVE.of(connected))
        }
    }
}

/// The label of the badge that says how much a finding asks of the user.
pub(crate) const fn network_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Attention => "Atenção",
        Severity::Info => "Informação",
        Severity::Good => "OK",
    }
}

/// Says one conclusion of the analysis in a sentence.
pub(crate) fn network_finding(finding: &Finding) -> String {
    match finding {
        Finding::ProxyDiscoveryUnanswered => "A descoberta automática de proxy está ligada, mas nenhum proxy respondeu nesta rede. Cada conexão nova espera por essa procura.".to_owned(),
        Finding::ProxyDiscoveryOn => "A descoberta automática de proxy está ligada: o macOS procura um proxy antes de cada conexão nova.".to_owned(),
        Finding::SlowDns { current, fastest } => format!(
            "O DNS em uso responde em {}. O mais rápido medido daqui responde em {}.",
            millis(*current),
            millis(*fastest)
        ),
        Finding::PacketLoss { sent, lost } => format!(
            "{lost} de {sent} pacotes enviados à internet ficaram sem resposta. Perda de pacotes trava chamadas e deixa tudo mais lento."
        ),
        Finding::NoPacketLoss { sent } => {
            format!("Nenhum pacote perdido em {sent} envios até a internet.")
        }
        Finding::SlowAnswer { latency } => format!(
            "A internet demora {} para responder. Acima de 100 ms, chamadas e jogos ficam com atraso.",
            millis(*latency)
        ),
        Finding::WifiSignal {
            quality,
            signal_dbm,
            band,
            generation,
        } => {
            let strength = match quality {
                SignalQuality::Strong => "Sinal Wi-Fi forte",
                SignalQuality::Fair => "Sinal Wi-Fi regular",
                SignalQuality::Weak => "Sinal Wi-Fi fraco",
            };
            let band = band.map_or_else(String::new, |band| format!(", em {}", band.label()));
            let generation =
                generation.map_or_else(String::new, |name| format!(", no padrão {name}"));
            let closer = if *quality == SignalQuality::Weak {
                " Mais perto do roteador a velocidade sobe."
            } else {
                ""
            };
            format!("{strength} ({signal_dbm} dBm){band}{generation}.{closer}")
        }
        Finding::VpnConnected => {
            "Uma VPN está conectada: tudo faz um caminho mais longo e a velocidade cai.".to_owned()
        }
        Finding::HeavyProgram {
            name,
            direction,
            rate,
            share_percent,
        } => match direction {
            Direction::Upload => format!(
                "{name} está enviando {rate}: {share_percent}% de tudo o que este Mac envia agora."
            ),
            Direction::Download => format!(
                "{name} está recebendo {rate}: {share_percent}% de tudo o que este Mac recebe agora."
            ),
        },
        Finding::Responsiveness { level, rpm } => {
            let level_name = network_responsiveness_level(*level);
            match level {
                ResponsivenessLevel::Low => format!(
                    "A conexão trava com a rede ocupada ({rpm} RPM, resposta {level_name}). Chamadas de vídeo sofrem durante um download."
                ),
                ResponsivenessLevel::Medium | ResponsivenessLevel::High => format!(
                    "A conexão responde bem com a rede ocupada ({rpm} RPM, resposta {level_name})."
                ),
            }
        }
    }
}

/// Says what only the user can do for the connection.
pub(crate) fn network_advice(advice: &Advice) -> String {
    match advice {
        Advice::CrowdedChannel { channel, neighbors } => format!(
            "O canal {channel} é dividido com {neighbors} redes vizinhas. No roteador, teste outro canal."
        ),
        Advice::SlowBand => "Este Mac está na faixa de 2,4 GHz. A rede de 5 GHz do mesmo roteador é bem mais rápida.".to_owned(),
        Advice::VpnProfiles { installed } => {
            const PROFILES: Counted =
                Counted::new("perfil de VPN instalado", "perfis de VPN instalados");
            format!(
                "Há {}. Uma VPN ligada costuma reduzir a velocidade: ligue só quando precisar.",
                PROFILES.of(*installed)
            )
        }
        Advice::PreferCable => {
            "Para chamadas de vídeo e jogos, o cabo de rede é mais estável que o Wi-Fi.".to_owned()
        }
    }
}

/// The name of an optimization.
pub(crate) const fn network_optimization_title(kind: &OptimizationKind) -> &'static str {
    match kind {
        OptimizationKind::DisableProxyDiscovery => "Desligar a descoberta automática de proxy",
        OptimizationKind::FlushDnsCache => "Limpar o cache de DNS",
        OptimizationKind::UseFasterDns { .. } => "Usar o DNS mais rápido medido",
    }
}

/// What an optimization changes and what to expect from it, in one line.
pub(crate) fn network_optimization_detail(optimization: &Optimization) -> String {
    let service = optimization.change.service().unwrap_or_default();
    match &optimization.kind {
        OptimizationKind::DisableProxyDiscovery => {
            format!("{service} · conexões novas deixam de esperar pela procura de um proxy")
        }
        OptimizationKind::FlushDnsCache => {
            "corrige sites presos em endereço antigo · nada é apagado do disco".to_owned()
        }
        OptimizationKind::UseFasterDns {
            provider,
            current,
            fastest,
        } => format!(
            "{service} · {provider}: {} (atual: {}) · filtros e nomes internos do DNS atual se perdem",
            millis(*fastest),
            millis(*current)
        ),
    }
}

/// Says what one change does to the settings, for the confirmation.
pub(crate) fn network_change(change: &NetworkChange) -> String {
    match change {
        NetworkChange::FlushDnsCache => "Cache de DNS: esvaziar".to_owned(),
        NetworkChange::SetProxyAutoDiscovery {
            service,
            is_enabled,
        } => format!(
            "Descoberta automática de proxy do {service}: {}",
            if *is_enabled { "ligar" } else { "desligar" }
        ),
        NetworkChange::SetDnsServers { service, servers } if servers.is_empty() => {
            format!("DNS do {service}: voltar ao que a rede informa")
        }
        NetworkChange::SetDnsServers { service, servers } => format!(
            "DNS do {service}: {}",
            servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Says what macOS will ask for before the changes are applied.
pub(crate) const fn network_authentication(kind: AuthenticationKind) -> &'static str {
    match kind {
        AuthenticationKind::TouchId => {
            "O macOS vai pedir a sua impressão digital ou a senha de administrador."
        }
        AuthenticationKind::Password => "O macOS vai pedir a senha de administrador.",
    }
}

/// Says why the network settings were not changed.
pub(crate) fn network_privilege_error(error: &PrivilegeError) -> String {
    administrator_failure("Os ajustes de rede não foram alterados", error)
}

/// Says why the traffic could not be read.
pub(crate) fn traffic_probe_error(error: &TrafficProbeError) -> String {
    match error {
        TrafficProbeError::Unavailable(reason) => {
            format!("O macOS não informou o tráfego de rede: {reason}")
        }
        TrafficProbeError::TimedOut => "A leitura do tráfego de rede demorou demais.".to_owned(),
        TrafficProbeError::Unreadable(reason) => {
            format!("O tráfego de rede veio num formato desconhecido: {reason}")
        }
    }
}

/// Says why the connection could not be analyzed.
pub(crate) fn inspection_error(error: &InspectionError) -> String {
    match error {
        InspectionError::Offline => "Este Mac não está conectado a uma rede.".to_owned(),
        InspectionError::Unavailable(reason) => {
            format!("Não foi possível ler os ajustes de rede: {reason}")
        }
    }
}

/// Says why the speed test produced no result.
pub(crate) fn speed_test_error(error: &SpeedTestError) -> String {
    match error {
        SpeedTestError::Unavailable(reason) => {
            format!("O teste de velocidade do macOS não está disponível: {reason}")
        }
        SpeedTestError::TimedOut => {
            "O teste de velocidade demorou demais e foi interrompido.".to_owned()
        }
        SpeedTestError::Failed(reason) => {
            format!("O teste de velocidade não terminou: {reason}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::age;

    const DAY: i64 = 86_400;

    #[test]
    fn says_how_long_ago_in_the_largest_fitting_unit() {
        assert_eq!(age(3_600), "hoje");
        assert_eq!(age(DAY), "há 1 dia");
        assert_eq!(age(14 * DAY), "há 14 dias");
        assert_eq!(age(45 * DAY), "há 1 mês");
        assert_eq!(age(200 * DAY), "há 6 meses");
        assert_eq!(age(800 * DAY), "há 2 anos");
        assert_eq!(age(-50), "hoje");
    }
}

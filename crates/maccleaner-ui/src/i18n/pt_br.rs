//! Brazilian Portuguese texts.

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::{EngineError, PrivilegeError};
use maccleaner_domain::applications::LeftoverKind;
use maccleaner_domain::docker::ResourceKind;
use maccleaner_domain::photos::SimilarityTolerance;
use maccleaner_domain::scan::{CloudState, FileCategory};

pub(crate) use maccleaner_domain::shared::group_digits;

pub(crate) const BRAND: &str = "MACCLEANER";

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

pub(crate) const TITLE_DASHBOARD: &str = "Central de operações";
pub(crate) const TITLE_CLEANUP: &str = "Limpeza";
pub(crate) const TITLE_APPS: &str = "Desinstalar aplicativos";
pub(crate) const TITLE_DUPLICATES: &str = "Arquivos duplicados";
pub(crate) const TITLE_PHOTOS: &str = "Fotos parecidas";
pub(crate) const TITLE_DEVELOPER: &str = "Artefatos de build";
pub(crate) const TITLE_DOCKER: &str = "Resíduos do Docker";
pub(crate) const TITLE_DISK_MAP: &str = "Mapa de setores";
pub(crate) const TITLE_MONITOR: &str = "Telemetria";

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
pub(crate) const ITEMS: &str = "itens";
pub(crate) const CANCEL: &str = "Cancelar";
pub(crate) const OPEN_FULL_DISK_ACCESS: &str = "Abrir ajustes de Acesso Total ao Disco";
pub(crate) const FULL_DISK_ACCESS_MISSING: &str =
    "Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises.";
pub(crate) const SCAN_LOST: &str = "A varredura parou por um erro interno. Tente de novo.";

pub(crate) const TRY_AGAIN: &str = "Tentar de novo";
pub(crate) const CLOSE: &str = "Fechar";
pub(crate) const SELECTED: &str = "selecionados";
pub(crate) const SELECT_SOMETHING: &str = "Selecione ao menos um item.";
pub(crate) const EMPTY: &str = "vazio";
pub(crate) const RULES: &str = "regras";
pub(crate) const RISK_SAFE: &str = "Seguro";
pub(crate) const RISK_REVIEW: &str = "Revisar";

// Removal dialogs, shared by every screen that deletes
pub(crate) const CONFIRM_TITLE: &str = "Autorização requerida";
pub(crate) const REMOVING_TITLE: &str = "Removendo";
pub(crate) const REMOVING_BODY: &str = "Cada item é conferido de novo antes de ser removido.";
pub(crate) const DONE_TITLE: &str = "Concluído";
pub(crate) const FREED: &str = "liberados";
pub(crate) const ITEMS_REMOVED: &str = "itens removidos · registrados no histórico";
pub(crate) const SENT_TO_TRASH: &str = "enviados para a Lixeira";
pub(crate) const ITEMS_SKIPPED: &str =
    "itens deixados como estavam: mudaram, sumiram ou o macOS não permitiu";
pub(crate) const ITEMS_UNRECORDED: &str = "remoções não puderam ser gravadas no histórico";

// Cleanup
pub(crate) const CLEANUP_ANALYZING: &str = "Analisando";
pub(crate) const CLEANUP_FAILED: &str = "Análise interrompida";
pub(crate) const CLEANUP_USER_GROUP: &str = "Conta do usuário";
pub(crate) const CLEANUP_USER_META: &str = "sem autenticação";
pub(crate) const CLEANUP_ADMIN_GROUP: &str = "Sistema";
pub(crate) const CLEANUP_ADMIN_META: &str = "pede Touch ID ou a senha do macOS";
pub(crate) const CLEANUP_ADMIN_UNAVAILABLE: &str = "só no app instalado · cargo xtask install";
pub(crate) const CLEANUP_CONFIRM_ADMIN: &str =
    "As áreas de sistema pedem Touch ID ou a senha de administrador, no diálogo do próprio macOS";
pub(crate) const ADMIN_PROMPT: &str = "O MacCleaner precisa de permissão de administrador para limpar as áreas de sistema que você selecionou.";
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
pub(crate) const APPS_COUNT: &str = "apps";
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
pub(crate) const APPS_ADMIN_META: &str = "pede Touch ID ou senha · apagado de vez";
pub(crate) const APPS_ACTION: &str = "Desinstalar";
pub(crate) const APPS_CHOOSE_FIRST: &str = "Escolha um app na lista.";
pub(crate) const APPS_CONFIRM_TRASH: &str =
    "Os itens da sua conta vão para a Lixeira e podem ser recuperados de lá";
pub(crate) const APPS_CONFIRM_ADMIN: &str =
    "Os itens de sistema pedem Touch ID ou senha e são apagados de vez";
pub(crate) const APPS_CONFIRM_REVIEW: &str =
    "Itens com o selo Revisar podem ser de outro app: confira antes";
pub(crate) const APPS_ADMIN_PROMPT: &str = "O MacCleaner precisa de permissão de administrador para remover os itens de sistema do app que você está desinstalando.";

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
pub(crate) const DOCKER_RECORDS: &str = "registros";
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
pub(crate) const DOCKER_SKIPPED: &str =
    "itens recusados pelo Docker: estavam em uso ou ele não respondeu";

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
pub(crate) const fn docker_kind_plural(kind: ResourceKind) -> (&'static str, &'static str) {
    match kind {
        ResourceKind::BuildCache => ("Cache de build", DOCKER_RECORDS),
        ResourceKind::Image => ("Imagens", "imagens"),
        ResourceKind::Container => ("Containers", "containers"),
        ResourceKind::Volume => ("Volumes", "volumes"),
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
pub(crate) const DEV_INTRO_BODY: &str = "Escolha onde ficam os seus projetos. O MacCleaner procura pastas que as ferramentas recriam sozinhas, como node_modules, target e bin/obj.";
pub(crate) const DEV_INTRO_NOTE: &str = "Uma pasta só é oferecida quando o arquivo do projeto está ao lado dela e nada ali é versionado no git.";
pub(crate) const DEV_SEARCHING: &str = "Procurando projetos";
pub(crate) const DEV_LIST_TITLE: &str = "Pastas recriáveis";
pub(crate) const DEV_NOTHING_FOUND: &str = "Nenhuma pasta de build encontrada aqui.";
pub(crate) const DEV_SEARCH_AGAIN: &str = "Procurar de novo";
pub(crate) const DEV_TOUCHED: &str = "alterado";
pub(crate) const DEV_ACTION: &str = "Excluir selecionados";
pub(crate) const DEV_AUTHORIZE: &str = "Autorizar exclusão";
pub(crate) const DEV_FOLDERS: &str = "pastas";
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
pub(crate) const DUP_COPIES: &str = "cópias";
pub(crate) const DUP_EACH: &str = "cada";
pub(crate) const DUP_FILES: &str = "arquivos";
pub(crate) const DUP_MORE_PREFIX: &str = "Mais";
pub(crate) const DUP_MORE_SUFFIX: &str =
    "grupos menores não são mostrados. Remova os maiores e procure de novo.";
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
pub(crate) const PHOTOS_COMPARED: &str = "fotos comparadas.";
pub(crate) const PHOTOS_NONE_BODY: &str =
    "Nenhuma delas se parece com outra neste nível de semelhança.";
pub(crate) const PHOTOS_GROUP: &str = "fotos parecidas";
pub(crate) const PHOTOS_FREES: &str = "libera";
pub(crate) const PHOTOS_REFERENCE: &str = "REFERÊNCIA";
pub(crate) const PHOTOS_MATCH: &str = "MATCH";
pub(crate) const PHOTOS_REMOVE: &str = "Remover";
pub(crate) const PHOTOS_NO_PICTURE: &str = "sem prévia";
pub(crate) const PHOTOS_KEEP_ONE: &str = "Ao menos uma foto de cada grupo precisa ficar.";
pub(crate) const PHOTOS_MARKED: &str = "fotos marcadas";
pub(crate) const PHOTOS_COUNT: &str = "fotos";
pub(crate) const PHOTOS_MORE_SUFFIX: &str =
    "grupos menores não são mostrados. Remova estes e procure de novo.";
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
pub(crate) const MONITOR_CORES: &str = "núcleos";
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
pub(crate) const MAP_BLOCKED_FOLDERS: &str = "pastas sem acesso";

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
        ApplicationError::Io { path, source } => {
            format!("O macOS recusou o acesso a {}: {source}", path.display())
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
    const DAY: i64 = 86_400;
    let days = seconds.max(0) / DAY;
    let plural = |amount: i64, one: &str, many: &str| {
        format!("há {amount} {}", if amount == 1 { one } else { many })
    };
    match days {
        0 => "hoje".to_owned(),
        1..=29 => plural(days, "dia", "dias"),
        30..=364 => plural(days / 30, "mês", "meses"),
        _ => plural(days / 365, "ano", "anos"),
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

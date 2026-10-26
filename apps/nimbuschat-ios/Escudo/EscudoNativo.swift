// EscudoNativo.swift — camada nativa iOS do escudo (VSE1–VSE8, RSD1–RSD12).
//
// SITUAÇÃO: código de partida da Tarefa T2, escrito contra as APIs oficiais da Apple. NÃO foi compilado nem
// executado na máquina que gerou este pacote (sem Xcode). Aceite só com os testes de aparelho real do
// RESIDUOS_NO_APARELHO_E_LICOES.md e do vigia_de_sensores. Toda DECISÃO fica no núcleo Rust
// (vigia_de_sensores::decidir); este arquivo só LÊ sinais e EXECUTA o que o núcleo decidir.
//
// Limites do iOS, declarados: o sistema não informa se OUTRO app usa microfone ou câmera; só avisa quando o
// áudio ou a câmera DO NIMBUS são interrompidos por outro app. Por isso o Nimbus PREVINE: toma o microfone com
// sessão exclusiva (outro app gravando é interrompido pelo sistema) e, no iPhone, a câmera não pode ser usada por
// app em segundo plano. Não há como bloquear a captura de tela; só
// detectar e esconder. Serviços de acessibilidade, sobreposições e teclados de terceiros: o sistema impede
// ou o app recusa (abaixo).
import AVFoundation
import UIKit

/// Espelho do enum do núcleo (vigia_de_sensores::Sinal). A ponte da T1 converte um no outro.
enum Sinal { case sim, nao, sistemaImpede, desconhecido }

/// RSD1: teclados de terceiros nunca recebem o que se digita no Nimbus.
/// Colocar no AppDelegate:
///   func application(_ application: UIApplication,
///                    shouldAllowExtensionPointIdentifier id: UIApplication.ExtensionPointIdentifier) -> Bool {
///       return id != .keyboard
///   }
extension UITextField {
    func protegerDigitacao() {
        autocorrectionType = .no; spellCheckingType = .no; autocapitalizationType = .none
        smartQuotesType = .no; smartDashesType = .no; smartInsertDeleteType = .no
        textContentType = nil  // nada de sugestão de preenchimento do sistema
    }
}

// RSD2 / LIB1: o Nimbus nunca coloca conteúdo na área de transferência (não existe copiar mensagem); a regra
// nimbus-area-de-transferencia reprova qualquer escrita nela. RZ6: ao sair e ao trocar de app, limpa.
enum AreaDeTransferencia {
    static func limparAoSair() { UIPasteboard.general.items = [] }
}

/// RSD3: capa sobre o conteúdo quando o app perde o foco (a foto do seletor de apps sai só com a capa).
final class CapaDePrivacidade {
    private var capa: UIView?
    func cobrir(_ janela: UIWindow) {
        guard capa == nil else { return }
        let v = UIVisualEffectView(effect: UIBlurEffect(style: .systemMaterial))
        v.frame = janela.bounds; v.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        janela.addSubview(v); capa = v
    }
    func descobrir() { capa?.removeFromSuperview(); capa = nil }
}

/// RSD5: arquivos do Nimbus fora de qualquer backup e com proteção completa (só abrem com o aparelho desbloqueado).
enum Arquivos {
    static func proteger(_ url: URL) throws {
        var u = url
        var valores = URLResourceValues(); valores.isExcludedFromBackup = true
        try u.setResourceValues(valores)
        try FileManager.default.setAttributes([.protectionKey: FileProtectionType.complete], ofItemAtPath: url.path)
    }
}

/// VSE1–VSE8: lê os sinais oficiais. Cada mudança chama `aoMudar`; a leitura também é refeita a cada 5 s no
/// máximo (VALIDADE_DA_LEITURA_SEGUNDOS), senão o núcleo fecha tudo.
final class VigiaDeSensores {
    private(set) var microfoneDeOutroApp: Sinal = .nao
    private(set) var cameraDeOutroApp: Sinal = .nao
    private(set) var capturaDeTelaTirada = false
    private let aoMudar: () -> Void
    private var observadores: [NSObjectProtocol] = []

    init(aoMudar: @escaping () -> Void) { self.aoMudar = aoMudar }

    func ligar() {
        let nc = NotificationCenter.default
        observadores.append(nc.addObserver(forName: UIScreen.capturedDidChangeNotification, object: nil, queue: .main) { [weak self] _ in self?.aoMudar() })
        observadores.append(nc.addObserver(forName: UIApplication.userDidTakeScreenshotNotification, object: nil, queue: .main) { [weak self] _ in
            self?.capturaDeTelaTirada = true; self?.aoMudar() })
        // Outro app tomou o áudio do Nimbus (gravação ou chamada de outro app).
        observadores.append(nc.addObserver(forName: AVAudioSession.interruptionNotification, object: nil, queue: .main) { [weak self] n in
            let tipo = (n.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt).flatMap(AVAudioSession.InterruptionType.init)
            self?.microfoneDeOutroApp = (tipo == .began) ? .sim : .nao; self?.aoMudar() })
        observadores.append(nc.addObserver(forName: AVAudioSession.routeChangeNotification, object: nil, queue: .main) { [weak self] _ in self?.aoMudar() })
        // Outro app pegou a câmera que o Nimbus estava usando.
        observadores.append(nc.addObserver(forName: AVCaptureSession.wasInterruptedNotification, object: nil, queue: .main) { [weak self] n in
            let motivo = (n.userInfo?[AVCaptureSessionInterruptionReasonKey] as? Int).flatMap(AVCaptureSession.InterruptionReason.init)
            if motivo == .videoDeviceInUseByAnotherClient { self?.cameraDeOutroApp = .sim; self?.aoMudar() } })
        observadores.append(nc.addObserver(forName: AVCaptureSession.interruptionEndedNotification, object: nil, queue: .main) { [weak self] _ in
            self?.cameraDeOutroApp = .nao; self?.aoMudar() })
    }

    func desligar() { observadores.forEach(NotificationCenter.default.removeObserver); observadores = [] }

    /// iPhone — PREVENÇÃO, não só detecção: o Nimbus toma o microfone com sessão de áudio exclusiva (sem
    /// `.mixWithOthers`). Ao ativar uma sessão não misturável, o sistema interrompe a sessão não misturável de
    /// qualquer outro app, inclusive gravação em segundo plano. Se outro app tomar o microfone depois, o Nimbus
    /// recebe a interrupção (acima) e o núcleo fecha o microfone (VSE6). A CONFIRMAR em aparelho real (teste
    /// VSE6-iOS da T2): gravação de outro app com sessão misturável.
    func tomarMicrofoneComExclusividade() throws {
        let sessao = AVAudioSession.sharedInstance()
        try sessao.setCategory(.playAndRecord, mode: .voiceChat, options: [])
        try sessao.setActive(true)
        microfoneDeOutroApp = .nao
        aoMudar()
    }

    /// iPhone: app em segundo plano não pode usar a câmera (o sistema impede). No iPad com várias janelas, outro
    /// app em primeiro plano pode; ali vale a interrupção da sessão de câmera do Nimbus (acima).
    var cameraDeOutroAppPorPlataforma: Sinal {
        UIDevice.current.userInterfaceIdiom == .phone ? .sistemaImpede : cameraDeOutroApp
    }

    /// Tela gravada, espelhada ou transmitida. iOS 17+: estado de captura da cena; antes: UIScreen.isCaptured.
    func telaCapturada(_ cena: UIWindowScene) -> Sinal {
        if #available(iOS 17.0, *) { return cena.traitCollection.sceneCaptureState == .active ? .sim : .nao }
        return cena.screen.isCaptured ? .sim : .nao
    }

    /// Outra tela ligada ao aparelho (cabo, AirPlay como segunda tela).
    func telaExterna() -> Sinal {
        UIApplication.shared.connectedScenes.contains { $0.session.role == .windowExternalDisplayNonInteractive } ? .sim : .nao
    }

    /// Áudio saindo para fone, carro, caixa de som ou AirPlay.
    func audioExterno() -> Sinal {
        let externos: Set<AVAudioSession.Port> = [.bluetoothA2DP, .bluetoothHFP, .bluetoothLE, .airPlay, .HDMI, .carAudio, .usbAudio, .headphones]
        return AVAudioSession.sharedInstance().currentRoute.outputs.contains { externos.contains($0.portType) } ? .sim : .nao
    }

    /// No iOS não há serviço de acessibilidade de terceiros, nem sobreposição sobre outro app, e o teclado de
    /// terceiros é recusado pelo AppDelegate: o sistema impede os três.
    let acessibilidadeNaoConfiavel: Sinal = .sistemaImpede
    let sobreposicao: Sinal = .sistemaImpede
    let tecladoDeTerceiros: Sinal = .sistemaImpede

    /// Consumido pelo núcleo: o evento de captura vale uma vez.
    func consumirCapturaTirada() -> Bool { let v = capturaDeTelaTirada; capturaDeTelaTirada = false; return v }
}

/// A chave que abre o contêiner cifrado vive só no núcleo (memória travada e zerada). A ponte (T1) implementa
/// este contrato; o app nunca guarda cópia da chave em variável do Swift.
protocol NucleoTrancavel { func trancar() }

/// RSD15: a chave só existe com o app aberto e desbloqueado. Ao ir para o segundo plano, o núcleo zera a chave;
/// uma perícia no aparelho ligado encontra só o contêiner cifrado.
final class GuardiaoDaChave {
    private let nucleo: NucleoTrancavel
    private var observador: NSObjectProtocol?
    init(nucleo: NucleoTrancavel) {
        self.nucleo = nucleo
        observador = NotificationCenter.default.addObserver(forName: UIApplication.didEnterBackgroundNotification, object: nil, queue: .main) { [weak self] _ in
            self?.esquecerChaveAoSair()
        }
    }
    func esquecerChaveAoSair() { nucleo.trancar() }
}

/// RSD16: o Keychain do iPhone sobrevive à desinstalação do app. Na primeira execução (sem o contêiner cifrado do
/// Nimbus no disco), apaga qualquer item deixado por uma instalação anterior. Toda chave é guardada só neste
/// aparelho, só com ele desbloqueado, nunca em backup nem em outro aparelho.
enum ChavesDoAparelho {
    static let acessibilidade = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
    static func apagarChavesDeInstalacaoAnterior(conteinerExiste: Bool) {
        guard !conteinerExiste else { return }
        for classe in [kSecClassGenericPassword, kSecClassKey, kSecClassInternetPassword, kSecClassCertificate, kSecClassIdentity] {
            SecItemDelete([kSecClass as String: classe] as CFDictionary)
        }
    }
}

// RSD14: a camada nativa NÃO abre conexão de rede. Toda rede passa pelo núcleo (módulo `rede`: rotas, Tor,
// tráfego de batida constante), que não usa cache HTTP. Por isso não há cache, cookie nem credencial de rede em
// disco. A regra nimbus-rede-fora-da-rota reprova qualquer URLSession criada aqui.


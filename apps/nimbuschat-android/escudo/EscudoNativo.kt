// EscudoNativo.kt — camada nativa Android do escudo (VSE1–VSE8, RSD1–RSD12).
//
// SITUAÇÃO: código de partida da Tarefa T2, escrito contra as APIs oficiais do Android. NÃO foi compilado nem
// executado na máquina que gerou este pacote (sem Android SDK). Aceite só com os testes de aparelho real do
// RESIDUOS_NO_APARELHO_E_LICOES.md e do vigia_de_sensores. Toda DECISÃO fica no núcleo Rust
// (vigia_de_sensores::decidir); este arquivo só LÊ sinais e EXECUTA o que o núcleo decidir.
//
// Manifesto: android:allowBackup="false", android:dataExtractionRules apontando para regras vazias (RSD5),
// <uses-permission android:name="android.permission.DETECT_SCREEN_CAPTURE"/> (Android 14+) e
// <uses-permission android:name="android.permission.DETECT_SCREEN_RECORDING"/> (Android 15+).
// // Regras: nenhum SDK de terceiros; nenhum log com conteúdo; na dúvida, Sinal.DESCONHECIDO (o núcleo fecha).
package nimbus.escudo

import android.accessibilityservice.AccessibilityServiceInfo
import android.app.Activity
import android.content.Context
import android.hardware.camera2.CameraManager
import android.hardware.display.DisplayManager
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Build
import android.provider.Settings
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import android.view.MotionEvent
import android.view.WindowManager
import android.view.accessibility.AccessibilityManager
import android.view.inputmethod.EditorInfo
import android.widget.EditText

/** Espelho do enum do núcleo (vigia_de_sensores::Sinal). A ponte da T1 converte um no outro. */
enum class Sinal { SIM, NAO, SISTEMA_IMPEDE, DESCONHECIDO }

/** RSD3 + VSE3: toda janela com conteúdo é protegida antes de mostrar qualquer coisa. */
object Janela {
    fun proteger(activity: Activity) {
        activity.window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)            // captura e gravação saem pretas
        if (Build.VERSION.SDK_INT >= 33) activity.setRecentsScreenshotEnabled(false) // nada no seletor de apps
        activity.window.decorView.filterTouchesWhenObscured = true                  // toque sob sobreposição é descartado
    }
}

/** RSD1: campo de texto sem aprendizado do teclado, sem sugestões e sem preenchimento automático. */
fun EditText.protegerDigitacao() {
    imeOptions = imeOptions or EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING
    inputType = inputType or android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
    if (Build.VERSION.SDK_INT >= 26) importantForAutofill = android.view.View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS
}

// RSD2 / LIB1: o Nimbus nunca coloca conteúdo na área de transferência (não existe copiar mensagem); a regra
// nimbus-area-de-transferencia reprova qualquer escrita nela. RZ6: ao sair e ao trocar de app, limpa.
object AreaDeTransferencia {
    fun limparAoSair(context: Context) {
        if (Build.VERSION.SDK_INT >= 28) context.getSystemService(android.content.ClipboardManager::class.java).clearPrimaryClip()
    }
}

/**
 * VSE1–VSE8: lê os sinais oficiais do sistema. Cada mudança gera uma nova Leitura para o núcleo; além disso,
 * a leitura é refeita no máximo a cada VALIDADE_DA_LEITURA_SEGUNDOS (5 s), senão o núcleo fecha tudo.
 */
class VigiaDeSensores(private val context: Context, private val aoMudar: () -> Unit) {
    private val camera = context.getSystemService(CameraManager::class.java)
    private val audio = context.getSystemService(AudioManager::class.java)
    private val telas = context.getSystemService(DisplayManager::class.java)
    private val acessibilidade = context.getSystemService(AccessibilityManager::class.java)

    @Volatile var cameraDeOutroApp = Sinal.DESCONHECIDO; private set
    @Volatile var microfoneDeOutroApp = Sinal.DESCONHECIDO; private set
    @Volatile var capturaDeTelaTirada = false; private set
    @Volatile var telaCapturada = Sinal.SISTEMA_IMPEDE; private set   // FLAG_SECURE já aplicado (Janela.proteger)
    @Volatile var sobreposicao = Sinal.NAO; private set
    private val camerasIndisponiveis = mutableSetOf<String>()
    private var nossaCameraAberta = false

    private val cameraCallback = object : CameraManager.AvailabilityCallback() {
        override fun onCameraUnavailable(id: String) { camerasIndisponiveis += id; atualizarCamera() }
        override fun onCameraAvailable(id: String) { camerasIndisponiveis -= id; atualizarCamera() }
    }
    private fun atualizarCamera() {
        cameraDeOutroApp = if (camerasIndisponiveis.isNotEmpty() && !nossaCameraAberta) Sinal.SIM else Sinal.NAO
        aoMudar()
    }
    fun nossaCamera(aberta: Boolean) { nossaCameraAberta = aberta; atualizarCamera() }

    private val gravacaoCallback = object : AudioManager.AudioRecordingCallback() {
        override fun onRecordingConfigChanged(configs: MutableList<android.media.AudioRecordingConfiguration>) {
            // Android 10+: o próprio app vê as suas gravações; gravação de outro app aparece sem identificação.
            // Quando dois gravam juntos, o sistema silencia um deles: se o nosso for silenciado, há outro gravando.
            microfoneDeOutroApp = when {
                Build.VERSION.SDK_INT < 29 -> Sinal.DESCONHECIDO
                configs.any { it.isClientSilenced } -> Sinal.SIM
                else -> Sinal.NAO
            }
            aoMudar()
        }
    }

    fun ligar(activity: Activity) {
        camera.registerAvailabilityCallback(cameraCallback, null)
        audio.registerAudioRecordingCallback(gravacaoCallback, null)
        if (Build.VERSION.SDK_INT >= 34) {
            activity.registerScreenCaptureCallback(activity.mainExecutor) { capturaDeTelaTirada = true; aoMudar() }
        }
        if (Build.VERSION.SDK_INT >= 35) {
            val estado = activity.windowManager.addScreenRecordingCallback(activity.mainExecutor) { e ->
                telaCapturada = if (e == WindowManager.SCREEN_RECORDING_STATE_VISIBLE) Sinal.SIM else Sinal.SISTEMA_IMPEDE
                aoMudar()
            }
            telaCapturada = if (estado == WindowManager.SCREEN_RECORDING_STATE_VISIBLE) Sinal.SIM else Sinal.SISTEMA_IMPEDE
        }
    }

    fun desligar(activity: Activity) {
        camera.unregisterAvailabilityCallback(cameraCallback)
        audio.unregisterAudioRecordingCallback(gravacaoCallback)
    }

    /** Chamado pela Activity em dispatchTouchEvent: toque por baixo de outra janela = sobreposição. */
    fun aoTocar(e: MotionEvent) {
        val obscuro = e.flags and (MotionEvent.FLAG_WINDOW_IS_OBSCURED or MotionEvent.FLAG_WINDOW_IS_PARTIALLY_OBSCURED) != 0
        if (obscuro) { sobreposicao = Sinal.SIM; aoMudar() }
    }

    fun telaExterna(): Sinal = if (telas.displays.size > 1) Sinal.SIM else Sinal.NAO

    /** Serviços de acessibilidade ligados fora da lista de confiáveis (leitores de tela oficiais). */
    fun acessibilidadeNaoConfiavel(confiaveis: Set<String>): Sinal {
        val ligados = acessibilidade.getEnabledAccessibilityServiceList(AccessibilityServiceInfo.FEEDBACK_ALL_MASK)
        return if (ligados.any { it.resolveInfo.serviceInfo.packageName !in confiaveis }) Sinal.SIM else Sinal.NAO
    }

    /** Teclado ativo diferente dos teclados de sistema confiáveis. */
    fun tecladoDeTerceiros(confiaveis: Set<String>): Sinal {
        val atual = Settings.Secure.getString(context.contentResolver, Settings.Secure.DEFAULT_INPUT_METHOD) ?: return Sinal.DESCONHECIDO
        return if (atual.substringBefore('/') in confiaveis) Sinal.NAO else Sinal.SIM
    }

    /** Áudio saindo para fone, carro, caixa de som ou outro aparelho. */
    fun audioExterno(): Sinal {
        val externos = setOf(AudioDeviceInfo.TYPE_BLUETOOTH_A2DP, AudioDeviceInfo.TYPE_BLUETOOTH_SCO, AudioDeviceInfo.TYPE_HDMI,
                             AudioDeviceInfo.TYPE_USB_DEVICE, AudioDeviceInfo.TYPE_USB_HEADSET, AudioDeviceInfo.TYPE_WIRED_HEADPHONES)
        return if (audio.getDevices(AudioManager.GET_DEVICES_OUTPUTS).any { it.type in externos }) Sinal.SIM else Sinal.NAO
    }

    /** Consumido pelo núcleo: o evento de captura vale uma vez. */
    fun consumirCapturaTirada(): Boolean { val v = capturaDeTelaTirada; capturaDeTelaTirada = false; return v }
}

/** A chave que abre o contêiner cifrado vive só no núcleo (memória travada e zerada). A ponte (T1) implementa. */
interface NucleoTrancavel { fun trancar() }

/**
 * RSD15: a chave só existe com o app aberto. Ao sair do primeiro plano, o núcleo zera a chave; uma perícia no
 * aparelho ligado encontra só o contêiner cifrado. Registrar em ProcessLifecycleOwner.get().lifecycle.
 */
class GuardiaoDaChave(private val nucleo: NucleoTrancavel) : DefaultLifecycleObserver {
    override fun onStop(owner: LifecycleOwner) { esquecerChaveAoSair() }
    fun esquecerChaveAoSair() = nucleo.trancar()
}

/**
 * RSD16: chave do aparelho no chip seguro (StrongBox), só com o aparelho desbloqueado, nunca exportável.
 * Aparelho sem StrongBox: cair para o ambiente seguro (TEE) e registrar o nível no núcleo (integridade).
 */
object ChaveDoAparelho {
    fun parametros(apelido: String): KeyGenParameterSpec =
        KeyGenParameterSpec.Builder(apelido, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setIsStrongBoxBacked(true)
            .setUnlockedDeviceRequired(true)
            .build()
}


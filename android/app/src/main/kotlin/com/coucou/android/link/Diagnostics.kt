package com.coucou.android.link

import java.io.IOException
import java.net.HttpURLConnection
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException
import java.net.URL
import javax.net.ssl.SSLException

/** The questions "Can't connect?" asks, in the order that makes each answer meaningful. */
enum class DiagStep(val title: String) {
    NETWORK("Telefon tarmog‘i"),
    ADDRESS("Saqlangan manzil javob beradi"),
    DISCOVERY("Kompyuter o‘zini e’lon qiladi"),
    SECURE("Xavfsiz ulanish (mahkamlangan sertifikat)"),
    TOKEN("Bog‘lash kodi va versiya"),
    RELAY("Relay"),
}

sealed class DiagResult {
    /** [detail] says what was seen in a few words. */
    data class Ok(val detail: String) : DiagResult()
    /** [problem] is what is wrong; [hint] is what to try, calmly. */
    data class Problem(val problem: String, val hint: String) : DiagResult()
    /** Not asked, because a step before it already failed. */
    data class Skipped(val why: String) : DiagResult()
}

/** What the checks need from the phone; the real one is [LanDiagnostics], tests use fakes. */
interface DiagEnv {
    fun network(): DiagResult
    fun address(): DiagResult
    fun discovery(): DiagResult
    fun secure(): DiagResult
    fun token(): DiagResult
    /** Null when the pairing has no relay. */
    fun relay(): DiagResult?
}

object Diagnostics {
    /** Runs every check, reporting each as it finishes. A step whose premise failed is skipped, not guessed. */
    fun run(env: DiagEnv, onResult: (DiagStep, DiagResult) -> Unit = { _, _ -> }): List<Pair<DiagStep, DiagResult>> {
        val out = ArrayList<Pair<DiagStep, DiagResult>>()
        fun put(step: DiagStep, r: DiagResult) { out += step to r; onResult(step, r) }
        fun failed(step: DiagStep) = out.any { it.first == step && it.second !is DiagResult.Ok }

        put(DiagStep.NETWORK, safe { env.network() })
        put(DiagStep.ADDRESS, if (failed(DiagStep.NETWORK)) DiagResult.Skipped("Wi-Fi kerak") else safe { env.address() })
        put(DiagStep.DISCOVERY, if (failed(DiagStep.NETWORK)) DiagResult.Skipped("Wi-Fi kerak") else safe { env.discovery() })
        put(DiagStep.SECURE, if (failed(DiagStep.ADDRESS)) DiagResult.Skipped("Manzil javob berishi kerak") else safe { env.secure() })
        put(DiagStep.TOKEN, if (failed(DiagStep.ADDRESS) || failed(DiagStep.SECURE)) DiagResult.Skipped("Xavfsiz ulanish kerak") else safe { env.token() })
        try { env.relay() } catch (e: Exception) { broken(e) }?.let { put(DiagStep.RELAY, it) }
        return out
    }

    private fun broken(e: Exception) = DiagResult.Problem("Tekshiruvning o‘zi ishlamadi (${e.javaClass.simpleName})", "Qayta urinib ko‘ring.")

    private fun safe(f: () -> DiagResult): DiagResult = try { f() } catch (e: Exception) { broken(e) }

    /** "192.168.1.20" -> "192.168.x.x": enough to tell a home network from a hotspot, not enough to find a house. */
    fun maskHost(host: String): String {
        val parts = host.split(".")
        return if (parts.size == 4 && parts.all { p -> p.isNotEmpty() && p.all(Char::isDigit) }) "${parts[0]}.${parts[1]}.x.x" else "nom (yashirilgan)"
    }

    /**
     * The report the user can copy and send. It names the checks and what they found; it never contains the pairing code,
     * the certificate fingerprint, the computer's name, the full address or any key.
     */
    fun report(appVersion: String, androidVersion: String, device: String, results: List<Pair<DiagStep, DiagResult>>, hostMasked: String, via: String): String =
        buildString {
            appendLine("Coucou for Android $appVersion · Android $androidVersion · $device")
            appendLine("Kompyuter: $hostMasked · ulanish: $via")
            for ((step, r) in results) {
                appendLine(
                    when (r) {
                        is DiagResult.Ok -> "OK       ${step.title}: ${r.detail}"
                        is DiagResult.Problem -> "PROBLEM  ${step.title}: ${r.problem}"
                        is DiagResult.Skipped -> "SKIPPED  ${step.title}: ${r.why}"
                    },
                )
            }
        }.trimEnd()
}

/**
 * The real checks against one pairing. Plain JVM except what the phone supplies: whether Wi-Fi is up, and a way to look the computer
 * up on the network ([discover]). The pairing code is used only to say hello, as the app does; it is never put in a result.
 */
class LanDiagnostics(
    private val p: PairingPayload,
    private val wifiUp: () -> Boolean,
    private val onCellular: () -> Boolean,
    private val discover: () -> List<Found>,
    private val connector: Connector = PinnedTls.connector(p),
    private val relayCheck: (() -> DiagResult)? = null,
    private val timeoutMs: Int = 3_000,
) : DiagEnv {
    override fun network(): DiagResult = when {
        wifiUp() -> DiagResult.Ok("Wi-Fi ulangan")
        p.relay != null && onCellular() -> DiagResult.Ok("Mobil internet (relay’dan foydalanish mumkin; to‘g‘ridan-to‘g‘ri ulanish uchun uydagi Wi-Fi kerak)")
        else -> DiagResult.Problem("Telefon Wi-Fi tarmog‘iga ulanmagan", "Kompyuteringiz ulangan Wi-Fi’ga ulaning yoki relay’dan foydalanish uchun «Uydagi Wi-Fi’dan tashqarida» ni yoqing.")
    }

    override fun address(): DiagResult {
        val started = System.nanoTime()
        val s = Socket()
        return try {
            s.connect(InetSocketAddress(p.host, p.port), timeoutMs)
            DiagResult.Ok("${Diagnostics.maskHost(p.host)} manzili ${(System.nanoTime() - started) / 1_000_000} ms da javob beradi")
        } catch (_: SocketTimeoutException) {
            DiagResult.Problem(
                "${Diagnostics.maskHost(p.host)} javob bermayapti",
                "Kompyuter yoqilganmi, Coucou ishlayaptimi va telefon bilan bog‘lanish yoqilganmi? Bitta Wi-Fi’dami? Mehmon tarmog‘i yoki mijozlarni ajratish buni bloklaydi.",
            )
        } catch (_: IOException) {
            DiagResult.Problem("${Diagnostics.maskHost(p.host)} ga ulanib bo‘lmayapti", "Kompyuterning manzili o‘zgargan bo‘lishi mumkin: keyingi qatorga qarang yoki manzilni Bosh sahifada kiriting.")
        } finally {
            runCatching { s.close() }
        }
    }

    override fun discovery(): DiagResult {
        val found = Discovery.candidates(discover(), p)
        return when {
            found.isEmpty() -> DiagResult.Problem(
                "Kompyuter bu yerda o‘zini e’lon qilmayapti",
                "Ba’zi tarmoqlar buni bloklaydi. Saqlangan manzil javob bersa, e’tibor bermasangiz ham bo‘ladi; aks holda manzilni Bosh sahifada kiriting.",
            )
            found.any { it.host == p.host && it.port == p.port } -> DiagResult.Ok("Topildi, manzili saqlangani bilan bir xil")
            else -> DiagResult.Problem("${Diagnostics.maskHost(found.first().host)} da topildi, saqlangan manzilda emas", "Bosh sahifani oching: ulanmagan paytda telefon manzilni o‘zi yangilaydi.")
        }
    }

    override fun secure(): DiagResult = try {
        connector.connect(timeoutMs).close()
        DiagResult.Ok("Kompyuter sertifikati siz bog‘langan sertifikatning o‘zi")
    } catch (e: SSLException) {
        if (generateSequence<Throwable>(e) { it.cause }.any { it is java.security.cert.CertificateException }) {
            DiagResult.Problem("Bu siz bog‘langan kompyuter emas (sertifikati boshqa)", "Coucou qayta o‘rnatilgan bo‘lishi yoki bu boshqa kompyuter bo‘lishi mumkin. Qayta bog‘lang.")
        } else {
            DiagResult.Problem("Xavfsiz ulanishni o‘rnatib bo‘lmadi", "Qayta urinib ko‘ring. Yana takrorlansa, qayta bog‘lang.")
        }
    } catch (_: IOException) {
        DiagResult.Problem("Xavfsiz ulanishni ochib bo‘lmadi", "Manzil javob beradi, lekin u yerda Coucou yo‘q. Telefon bilan bog‘lanish yoqilganmi?")
    }

    override fun token(): DiagResult {
        val s = try {
            connector.connect(timeoutMs)
        } catch (_: IOException) {
            return DiagResult.Problem("Salomlashish uchun ulanib bo‘lmadi", "Yuqoridagi qatorlarga qarang.")
        }
        return try {
            s.soTimeout = 5_000
            s.getOutputStream().apply { write((Wire.encode(ClientMsg.Hello(Protocol.VERSION, p.token, "Diagnostics", emptyList())) + "\n").toByteArray()); flush() }
            val line = readLine(s) ?: return DiagResult.Problem("Kompyuter ulanishni darhol yopdi", "Qayta urinib ko‘ring.")
            when (val m = Wire.decodeServer(line)) {
                is ServerMsg.Welcome ->
                    if (m.version == Protocol.VERSION) DiagResult.Ok("Bog‘lash kodi qabul qilindi, protokol versiyasi ${m.version}")
                    else DiagResult.Problem("Kompyuter protokolining versiyasi ${m.version}, bu ilovaniki ${Protocol.VERSION}", "Kompyuterdagi Coucou’ni yoki ilovani yangilang.")
                is ServerMsg.Error ->
                    if (m.code == "auth") DiagResult.Problem("Bog‘lash kodi rad etildi", "Kompyuter yangi kod yaratgan («Qayta bog‘lash»). Uning yangi kodini skanerlang.")
                    else DiagResult.Problem("Kompyuter xato bilan javob berdi (${m.code})", "Qayta urinib ko‘ring.")
                else -> DiagResult.Problem("Salomlashishga kutilmagan javob", "Kompyuterdagi Coucou’ni va ilovani yangilang.")
            }
        } catch (_: SocketTimeoutException) {
            DiagResult.Problem("Salomlashishga javob yo‘q", "Kompyuter band yoki uxlab qolgan. Qayta urinib ko‘ring.")
        } catch (_: IOException) {
            DiagResult.Problem("Salomlashish paytida ulanish uzildi", "Qayta urinib ko‘ring.")
        } finally {
            runCatching { s.getOutputStream().apply { write((Wire.encode(ClientMsg.Bye) + "\n").toByteArray()); flush() } }
            runCatching { s.close() }
        }
    }

    override fun relay(): DiagResult? = relayCheck?.invoke()

    private fun readLine(s: Socket): String? {
        val b = java.io.ByteArrayOutputStream()
        while (b.size() < Protocol.MAX_LINE_BYTES) {
            val x = s.getInputStream().read()
            if (x < 0) return null
            if (x == '\n'.code) return b.toString("UTF-8")
            b.write(x)
        }
        return null
    }

    companion object {
        /**
         * The relay answers `GET /` with "Coucou link relay" without a key and without touching any room, so this check can never
         * replace the phone's live connection (joining a room as the phone would).
         */
        fun relayReachable(relay: RelayPairing, timeoutMs: Int = 5_000): DiagResult {
            val url = relay.relayUrl() ?: return DiagResult.Problem("Bog‘lanishdagi relay manzili yaroqsiz", "Qayta bog‘lang.")
            return try {
                val conn = URL("${if (url.tls) "https" else "http"}://${url.host}:${url.port}/").openConnection() as HttpURLConnection
                conn.connectTimeout = timeoutMs
                conn.readTimeout = timeoutMs
                conn.instanceFollowRedirects = false
                val body = try { conn.inputStream.bufferedReader().readText().take(64) } finally { conn.disconnect() }
                if (conn.responseCode == 200 && body.startsWith("Coucou link relay")) DiagResult.Ok("${url.host} dagi relay javob beradi")
                else DiagResult.Problem("${url.host} javob beradi, lekin bu Coucou relay’i emas", "Kompyuterdagi manzilni tekshiring («Uydagi Wi-Fi’dan tashqarida»).")
            } catch (_: IOException) {
                DiagResult.Problem("${url.host} dagi relay’ga ulanib bo‘lmayapti", "Internet ulanishingizni va relay manzilini tekshiring.")
            }
        }
    }
}

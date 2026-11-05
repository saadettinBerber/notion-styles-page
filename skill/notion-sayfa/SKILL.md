---
name: notion-page
description: Markdown notlarını Notion görünümlü, tek dosyalık bir HTML sayfasına çevirir. Sayfa internetsiz açılır ve doğrudan gönderilebilir. Kapak, ikon, etiketler, callout, toggle, sütunlar, gömülü video, SVG çizim, ==vurgu==, kod renklendirme ve açık/koyu tema içerir. Kullanıcı "Notion gibi bir sayfa", "bu notları sayfa yap", "paylaşılabilir HTML sayfası", "callout'lu / toggle'lı sayfa" dediğinde ya da bir .md dosyasını Notion görünümlü HTML'e çevirmek istediğinde kullan.
---

# Notion sayfası

Bir `.md` dosyası yazılır, `notion-page` onu Notion görünümlü, tek dosyalık bir `.html` sayfasına çevirir. Sayfa internet gerektirmez, videolar sayfanın içine gömülür, dosya doğrudan gönderilebilir.

Skill dizini `${CLAUDE_SKILL_DIR}`, aşağıda `$SKILL`:

```bash
SKILL="${CLAUDE_SKILL_DIR}"
```

## İlk kullanımda

Terminalde `notion-page --help` çalışmıyorsa Rust komutunu Cargo ile kur.

```bash
repo_root="$(cd -P "$SKILL" && cd ../.. && pwd)" && cargo install --locked --path "$repo_root"
```

## İçerik kullanıcınındır

Sayfanın anlatımı, örnekleri ve sırası kullanıcıdan gelir. Sen onu bu kalıba döker, biçimi düzeltirsin. İstenmemiş bölüm, açıklama ya da süs bloğu ekleme. Bir yer eksik ya da belirsizse sor.

Not oluştururken veya kısaltırken [../basic-make-notes.md](../basic-make-notes.md) rehberini uygula: sohbeti hatırlamayı gerektirmeyen, anlam kaybı olmadan öz notlar yaz.

## Rehberleri terminalden oku

`notion-page --skill` rehberleri birer cümleyle listeler. `notion-page --skill basic-make-notes` seçilen rehberin tamamını gösterir; kaynak klasörü gerekmez. Diğer adlar: `notion-page` bu skill, `yazim` Markdown kuralları, `cizim` SVG kuralları, `vitrin` örnek sayfa, `okuyucu` belge API'si. Yerel bağlantılar açılamıyorsa ilgili rehberi bu komutla oku.

## Akış

1. **Sayfayı yaz.** Her dosya `---` satırları arasında `title` ya da `baslik` içeren bir başlık bloğuyla başlar. Bütün yazım kuralları [references/yazim.md](references/yazim.md) içindedir. Bütün blokların canlı örneği [templates/vitrin.md](templates/vitrin.md) dosyasıdır: emin olmadığında ona bak, yeni bir sayfaya onu kopyalayarak da başlayabilirsin.
2. **Üret.**

   ```bash
   notion-page notes.md
   notion-page notes/*.md --output pages
   ```

   Her dosya için bir `✓` satırı görünür. `✗` satırı hatanın nedenini söyler: düzelt ve yeniden üret.

   Kullanıcı yalnızca HTML dosyasını saklamak isterse `notion-page --archive notes.md` kullan. Bu komut Markdown'ı HTML'ye gömer ve kaynak dosyayı HTML kaydedildikten sonra siler. `notion-page --restore notes.html` özgün Markdown'ı geri çıkarır. Hedef dosya zaten varsa `--output` ile başka bir dizin seç.
3. **Göster.** Sayfayı tarayıcıda aç ve kendin bak: taşan yazı, çizgiye binen etiket, açıkta kalmış bir `:::` satırı, koyu temada okunmayan bir şey, telefonda taşma. Sorun varsa düzeltip yeniden üret, sonra kullanıcıya göster. Video gömülü büyük sayfalar bazı tarayıcılarda `file://` ile açılmaz. O zaman sayfanın klasöründe `python3 -m http.server 8000` çalıştır ve `http://127.0.0.1:8000/sayfa.html` adresini aç.

## Hangi blok ne için

- **Callout:** öne çıkan fikir. Önerilen renkler: 💡 mavi fikir, ⚠️ sari dikkat, ✅ yesil sonuç, ❌ kirmizi yanlış bilinen.
- **Toggle:** ana akışı bölmeyen ayrıntı, "merak edene" notu.
- **Toggle heading:** başlığı büyük puntolu toggle. Az kullanılır, sayfada sürpriz ya da vurgulu bir kapanış içindir.
- **Sütunlar:** yan yana karşılaştırma. Telefonda alt alta dizilir.
- **Video:** okur başlatır. Kapak görseli neredeyse her zaman gerekir.
- **Çizim:** `<figure>` içinde gömülü bir `<svg>`. Kurallar ve üretme yolu [references/cizim.md](references/cizim.md) içindedir.

## Sık yapılan hatalar

- Sayfa başlığını gövdede `#` ile tekrar yazmak. Başlık `baslik` alanından gelir, bölümler `##` ile açılır.
- İç içe blokta dıştaki bloğa daha fazla iki nokta vermemek (`:::::` > `::::` > `:::`). Verilmezse sayfada açıkta bir `:::` satırı görünür.
- `<figure>` ile `</figure>` arasında boş satır bırakmak. Markdown boş satırda HTML'i keser ve çizim bozulur.
- Renk adını yanlış yazmak (`mavii`). Sözcük renk sayılmaz, ikon gibi görünür.
- Sayfayı tek dosya diye göndermek ama kapak görselini ya da `![](…)` görselini yerel bir yoldan vermek. Bunlar gömülmez, alıcıda görünmez. Ayrıntısı [references/yazim.md](references/yazim.md) → "Medya" bölümündedir.

# Sayfa yazımı

Bir sayfa tek bir `.md` dosyasıdır: üstte bir başlık bloğu, altında Markdown gövde. Bütün blokların canlı örneği [../templates/vitrin.md](../templates/vitrin.md) dosyasındadır.

## Başlık bloğu (zorunlu)

Her dosya `---` satırları arasındaki bir YAML bloğuyla başlar:

```yaml
---
baslik: Haftalık notlar
ikon: 🧠
kapak: mavi
tarih: 2026-09-27
etiketler: [toplantı, plan]
ozet: Bu haftanın kararları ve açık soruları.
---
```

| Alan | Zorunlu | Ne |
|---|---|---|
| `title` veya `baslik` | evet | Sayfanın adı. İki alan birlikteyse `baslik` kullanılır. Gövde bölümleri `##` ile başlar. |
| `ikon` | hayır | Başlığın üstündeki tek emoji. Varsayılan 📄. |
| `kapak` | hayır | Renk adı (aşağıdaki liste), görsel adresi (`https://…` ya da `./gorsel.png`) ya da `yok`. Boş bırakılırsa kapak olmaz. **Tanınmayan değer hata verir.** |
| `tarih` | hayır | `YYYY-AA-GG`. |
| `etiketler` | hayır | `[a, b]` ya da `a, b`. Aynı etiket her sayfada aynı renkte görünür. |
| `ozet` | hayır | Tek cümle. Sayfada görünmez, bağlantı önizlemesinde kullanılır. |
| `font` | hayır | `daily-dose` veya `system`. Varsayılan `daily-dose`; tanınmayan değer hata verir. |

**Renk adları:** `gri`, `kahverengi`, `turuncu`, `sari`, `yesil`, `mavi`, `mor`, `pembe`, `kirmizi`. Türkçe karakterli yazımlar (`sarı`, `yeşil`, `kırmızı`) ve İngilizce adlar (`blue`, `green`, …) da geçer.

## Yazı fontu

Varsayılan `daily-dose`, Daily Dose of Data Science'ın yazı fontlarından esinlenir:
gövde Noto Serif, başlıklar Domine kullanır. Normal, **kalın** ve *italik* metin ile
Türkçe karakterler desteklenir. Font dosyaları HTML'ye gömüldüğü için internet gerekmez.
`system`, önceki sistem fontlu görünümü kullanır. Kod her iki seçimde de eş aralıklı
fontla gösterilir.

Bir sayfanın seçimini başlık bloğuna yaz:

```yaml
font: system
```

Bir veya daha fazla dosyanın seçimini üretirken değiştirmek için
`notion-page notes.md --font daily-dose` kullan. Öncelik komuttaki `--font`, sonra
başlık bloğundaki `font`, sonra `daily-dose` varsayılanıdır. Arşivler de bu seçimi
destekler; `--restore` ile font seçilmez, özgün Markdown olduğu gibi geri çıkarılır.

## Gövde: standart Markdown

Başlıklar (`#`, `##`, `###`), **kalın**, *italik*, ~~üstü çizili~~, `satır içi kod`, bağlantı, görsel, madde ve numaralı liste, `- [ ]` / `- [x]` yapılacaklar, `>` alıntı, `---` ayırıcı, tablo, dil etiketli kod bloğu (```` ```python ````) ve dipnot (`[^1]`) çalışır. Kod blokları yana kayar ki girinti bozulmasın; `text` etiketli blok (```` ```text ````) ise düz metin sayılır, uzun satırı kutunun içinde alta indirir. İstem, günlük ya da uzun düz metin için onu kullan.

Ek olarak `==vurgulu metin==` yazılırsa metin sarı zeminle vurgulanır.

## Gövde: Notion blokları

Bloklar `:::ad` satırıyla açılır, tek başına duran `:::` satırıyla kapanır. İçlerine her türlü Markdown yazılabilir. Kod çitinde gösterilen `:::ad` yalnız örnek metindir.

[Belge okuyucusu](okuyucu.md) bilinmeyen veya kapanmamış kapsayıcıyı özgün satır numarasıyla reddeder.

### Callout: öne çıkan kutu

```markdown
:::callout 💡 mavi
Asıl fikir burada.
:::
```

`callout` sözcüğünden sonra bir emoji ve bir renk adı gelir. İkisi de isteğe bağlıdır, sıraları önemsizdir. Varsayılan 💡 ve gridir.

> Renk adında yazım hatası yapılırsa (`mavii`) o sözcük renk sayılmaz ve ikon gibi görünür. Renk adını yukarıdaki listeden kopyala.

### Toggle: açılır kapanır bölüm

```markdown
:::toggle Merak edene: biraz daha ayrıntı
Başta gizli duran ayrıntı.
:::
```

`toggle` sözcüğünden sonraki metin tıklanan başlıktır. Başlıkta **kalın** ve `kod` kullanılabilir.

Başlığın büyük puntoyla dikkat çekmesi istenirse `:::toggle-heading` kullanılır: Notion'daki "toggle heading". Yazımı `:::toggle` ile aynıdır, yalnız başlık `#` başlık boyutunda görünür.

### Sütunlar: yan yana karşılaştırma

```markdown
::::columns
:::column
**Eski yol**

Kurallar elle yazılır.
:::
:::column
**Yeni yol**

Kurallar veriden öğrenilir.
:::
::::
```

Telefonda sütunlar alt alta dizilir.

### Video: sayfaya gömülü, okur başlatır

```markdown
:::video medya/tanitim.mp4 medya/tanitim-kapak.jpg
Videonun alt yazısı.
:::
```

- İlk yol videodur (`.mp4`, `.webm`). İkinci yol isteğe bağlı **kapak görselidir** (`.jpg`, `.png`, `.webp`).
- Yollar `.md` dosyasının bulunduğu klasöre göre yazılır. Medyayı sayfanın yanında bir `medya/` klasöründe tutmak düzenli olur.
- Video kendiliğinden başlamaz. Okur oynat düğmesine basana kadar kapak görseli görünür. Videonun ilk karesi çoğu zaman boş olduğu için kapak neredeyse her zaman gerekir. Kapak, videodan anlamlı bir kare seçilerek üretilir: `ffmpeg -ss 3 -i medya/ad.mp4 -frames:v 1 -vf scale=1280:-1 -q:v 3 medya/ad-kapak.jpg`.
- Bloğun gövdesi videonun alt yazısıdır ve boş bırakılabilir.
- Dosya yoksa ya da türü yanlışsa üretim `✗` ile durur.

### ⚠️ İç içe blok kuralı

Bir blok başka bir bloğun içindeyse **dıştaki blok daha fazla iki nokta** kullanır: dışta `::::`, içte `:::`. Üç düzey iç içe olursa en dışta `:::::` kullanılır. Aynı sayıda iki nokta kullanılırsa ilk `:::` iki bloğu birden kapatır ve sayfada açıkta bir `:::` satırı görünür. Callout içine toggle koyarken de aynı kural geçerlidir:

```markdown
::::callout 💡 mavi
Ana fikir.

:::toggle Ayrıntı
Gizli açıklama.
:::
::::
```

## Medya: ne gömülür, ne gömülmez

| Ne | Gömülür mü | Not |
|---|---|---|
| `:::video` bloğundaki video ve kapak görseli | evet | Sayfa tek dosya kalır ama medyanın boyutu kadar büyür. |
| `<figure>` içindeki SVG çizim | evet | Metnin kendisidir. Kurallar [cizim.md](cizim.md) içindedir. |
| `kapak:` alanındaki görsel | hayır | Adresiyle bağlanır. `./gorsel.png` gibi bir yol **HTML dosyasına göre** çözülür. |
| Markdown görseli `![açıklama](gorsel.png)` | varsayılanda hayır | Adresiyle bağlanır, yol yine HTML dosyasına göre çözülür. Kütüphane çağıranı güvenli gömmeyi açıkça seçebilir. |

Sayfa tek dosya olarak gönderilecekse gömülmeyen görseller `https://` adresiyle verilir ya da çizim SVG olarak yapılır. `-o` ile başka bir klasöre üretilen sayfada yerel görsel yolları o klasöre göre çözülür.

Kütüphane API'sinde `prepared.render(media, embed_images=media.embedded_image_data_uri)`
belge klasörü içindeki düzenli PNG, JPEG, GIF ve WebP dosyalarını gömer. `media`, belge
klasörüyle kurulmuş `MediaFolder` olmalıdır. Tür dosya uzantısından değil gerçek
görsel baytlarından belirlenir; kesik dosya, SVG, `../` ve sembolik bağ reddedilir.
Üretilen `img` öğesi kaynak yolunu `data-source` özniteliğinde tutar. Aynı görselin iki
kullanımı iki ayrı `img` olarak kalır. Seçenek yalnızca gövde Markdown'ındaki görsellere
uygulanır. API ve hazırlama sırası [okuyucu.md](okuyucu.md) içindedir.

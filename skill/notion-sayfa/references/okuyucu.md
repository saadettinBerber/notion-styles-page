# Kaynak aralıklarını koruyan Rust okuyucusu

Python bağlayıcısının kurulumu Cargo ve Python 3.10 veya üstünü gerektirir.
Depo kökünde çalıştır.

```sh
python -m pip install ./bindings/python
```

Rust'ta giriş noktası `notion_sayfa::prepare_document` işlevi, Python'da
`notionmd_rust.MarkdownReader` sınıfıdır. Python örneği:

```python
from pathlib import Path

from notionmd_rust import MarkdownReader, MediaFolder, Topic, load_page_template

path = Path("notlar.md")
source = path.read_bytes().decode("utf-8")
media = MediaFolder(path.parent)
prepared = MarkdownReader(media).prepare(source, id_prefix="tr-")
document = prepared.document()
# Medya ve içerik politikasını burada document üzerinden uygula.
rendered = prepared.render()
if document.topic is not None:
    topic = Topic(body="", **vars(document.topic))
    page_html = load_page_template().render(topic, rendered.html, lang="tr")
```

`prepare()` başlıklı bir sayfayı veya başlıksız bir Markdown parçasını ayrıştırır.
`document()` kaynak aralıklarını ve sorgu sonuçlarını verir. İki adımda da medya
okunmaz. `render()` aynı hazırlanmış belgeden HTML gövdesi ve çizim ağacı
üretir. İsteğe bağlı `media` argümanıyla belge başka bir medya sağlayıcısıyla
yeniden çizilebilir. Sağlayıcı `video_data_uri(path)` ve
`image_data_uri(path)` işlemlerini sunar.

## Kaynak koordinatları ve sorgular

Aralıkların `start` ve `end` alanları özgün Unicode metnin kod noktası
indisleridir. Bitiş indisi hariçtir. `source[span.start:span.end]` özgün dilimi
verir. BOM ve CRLF korunacaksa örnekteki gibi dosyayı bayt olarak oku.

- `topic` YAML başlık verisini, `header_fields` alan adlarını ve özgün değer
  aralıklarını taşır. Başlıksız parçada `topic` değeri `None` olur.
- `blocks` blok türünü, kaynak aralığını, yapı imzasını, paylaşılabilirlik
  işaretini ve varsa çocukları taşır. Toggle başlığı `title` ve
  `title_range` alanlarından okunur.
- `image_destinations` çözülmüş görsel hedeflerini ve aralıklarını,
  `container_uses` Notion kapsayıcılarını ve aralıklarını verir.
- `protected_ranges` kod ve bağlantı hedefi gibi parçaları kaynak aralığıyla
  işaretler. `plain_text` metin sorgusudur.
- `formatting`, `content_text`, özgün kod baytlarını içeren `code_signatures`,
  `raw_html_ranges` ve `container_titles` alanlarını taşır.

Hatalı YAML veya bilinmeyen ya da kapanmamış bir Notion kapsayıcısı için
Python bağlayıcısı `ValueError` verir. Kapsayıcı hatasında özgün satır numarası
mesaja eklenir.

## HTML ve medya

`rendered.html` HTML gövdesidir. `rendered.render_nodes` çift dilli yerleşim
gibi işlemler için çizim ağacını verir. Yapraklar `html`, `shared` ve
`inline` alanlarını taşır. Dallar açılış ve kapanış HTML'ini ve çocuklarını
taşır. Dipnot dallarında kimlik ve sıra bilgisi de bulunur.

`id_prefix`, aynı sayfaya yerleştirilen belgelerin üretilmiş başlık ve dipnot
kimliklerini ayırır. Örneğin `en-` öneki ilk dipnot için `fn-en--1` üretir.
`MarkdownReader(media, allow_raw_html=False)` ham HTML'yi kaçışlı metin olarak
çizer. Kaynak aralıkları çizim seçeneklerinden bağımsızdır.

`prepared.render(embed_images=media.embedded_image_data_uri)` ile gövde
görselleri açıkça gömülebilir. Çözümlenmiş hedef callback'e verilir ve
`data-source` özniteliğinde korunur. Rust medya katmanı, belge dizini içindeki
düzenli PNG, JPEG, GIF ve WebP dosyalarının baytlarını ve varsa animasyon
karelerini doğrular. `MediaFolder` gömülecek dosyada sembolik bağ,
mutlak yol ve üst dizin geçişini reddeder. `inspect_image(content)`, dosya
açmadan aynı doğrulamayı uygular ve `ImageInfo(media_type, width, height)`
döndürür. Medya erişimine izin verme kararı çağırana aittir.

`load_page_template().render(topic, body_html, lang="tr", extra_styles="", font=None)`,
verilen gövdeyi sayfa şablonuna yerleştirir. `font`, `"daily-dose"` veya
`"system"` seçeneğidir. Varsayılan `daily-dose`, Noto Serif gövde ve Domine
başlık fontlarını HTML'e gömer; sayfa internetsiz de aynı yazı tipleriyle açılır.
`system`, işletim sisteminin yazı tiplerini ve önceki sayfa ölçülerini kullanır.
Kod fontu bu seçimden bağımsızdır. `extra_styles` en son uygulanır.

YAML başlığındaki `font:` değeri `document.topic.font` üzerinden taşınır.
Alan belirtilmezse değeri `None` olur. `Topic` kurucusunun son, isteğe bağlı
argümanı da `font=None` olduğundan yukarıdaki `Topic(body="", **vars(document.topic))`
aktarımı font seçimini korur. Font alanı diğer `Topic` alanları gibi değişmezdir.
Şablona açıkça verilen `font`, konunun seçiminden önceliklidir; ikisi de `None`
ise `daily-dose` kullanılır. Bilinmeyen font adı `ValueError`, Python çağrısında
metin dışındaki bir font değeri `TypeError` verir. Örneğin:

```python
topic = Topic("Notlar", "", font="system")
page_html = load_page_template().render(topic, rendered.html, font="daily-dose")
```

Rust'ta aynı seçim `PageOptions { font: Some(FontPreset::System) }` ile
`render_page_with_options` veya `render_shell_with_options` işlevine verilir.
Mevcut `render_page` ve `render_shell` işlevleri belge seçimini ve ortak
varsayılanı kullanmaya devam eder. Font seçimi Markdown gövdesini, belge
sorgularını veya medya sağlayıcısını değiştirmez.

`writing_guide()`, Rust'a gömülen
[kanonik yazım kılavuzunu](yazim.md) döndürür.

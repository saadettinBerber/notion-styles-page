---
title: Şablon Vitrini
ikon: 🧪
kapak: mor
tarih: 2026-10-07
etiketler: [şablon, örnek]
ozet: Dönüştürücünün desteklediği bütün blokların örnek sayfası.
---

Bu sayfa gerçek bir sayfa değil. Dönüştürücünün her bloğu nasıl çizdiğini görmek için hazırlandı. Yeni bir sayfaya bu dosyayı kopyalayarak başlanabilir.

:::callout 💡 mavi
**Callout** kutusu önemli bir fikri öne çıkarmak için kullanılır. İçine **kalın**, *italik*, `kod` ve [bağlantı](https://example.com) yazılabilir.
:::

# Başlık 1

Düz paragraf. ==Vurgulu metin== böyle görünür, ~~üstü çizili~~ metin de böyle.

## Başlık 2

- Madde işaretli liste
- İkinci madde
  - İç içe madde

1. Numaralı liste
2. İkinci adım

### Başlık 3

- [x] Tamamlanmış görev
- [ ] Bekleyen görev

> Alıntı bloğu. Notion'daki gibi solda kalın bir çizgiyle görünür.

:::callout ⚠️ sari
Uyarı tonu için sarı callout.
:::

:::callout ✅ yesil
Olumlu sonuç için yeşil callout.
:::

:::toggle Açılır kapanır bölüm (tıkla)
Toggle'ın içi başta gizlidir. Detay, ek açıklama veya "merak edene" notları için uygundur.

- İçine liste de yazılabilir.
:::

:::toggle-heading Büyük başlıklı toggle
Notion'daki "toggle heading". Yazımı toggle ile aynıdır, yalnız başlık `#` boyutunda görünür. Az kullanılır: sayfada sürpriz ya da vurgulu bir kapanış içindir.
:::

::::callout 🧭 mor
İç içe blokta dıştaki blok daha fazla iki nokta kullanır.

:::toggle Callout içinde toggle
Dışta `::::`, içte `:::`.
:::
::::

::::columns
:::column
**Sol sütun**

Karşılaştırma yaparken iki sütun kullanışlıdır.
:::
:::column
**Sağ sütun**

Telefonda sütunlar alt alta dizilir.
:::
::::

---

<figure>
<svg viewBox="0 0 680 140" role="img" aria-label="Soldan sağa: notlar.md dosyası uret ile tek dosyalık sayfa.html sayfasına çevrilir.">
<rect x="40" y="40" width="160" height="56" rx="10" class="diagram-box" stroke-width="1.5"/>
<text x="120" y="74" text-anchor="middle" font-size="15" class="diagram-text">notlar.md</text>
<line x1="200" y1="68" x2="250" y2="68" class="diagram-line" stroke-width="2"/>
<polygon points="250,62 260,68 250,74" class="diagram-muted"/>
<rect x="260" y="40" width="160" height="56" rx="10" class="diagram-node" stroke-width="1.5"/>
<text x="340" y="74" text-anchor="middle" font-size="15" font-weight="600" class="diagram-text">uret</text>
<line x1="420" y1="68" x2="470" y2="68" class="diagram-line" stroke-width="2"/>
<polygon points="470,62 480,68 470,74" class="diagram-muted"/>
<rect x="480" y="40" width="160" height="56" rx="10" class="diagram-accent-line" stroke-width="2"/>
<text x="560" y="74" text-anchor="middle" font-size="15" font-weight="600" class="diagram-accent">sayfa.html</text>
<text x="120" y="124" text-anchor="middle" font-size="13" class="diagram-muted">yazdığın metin</text>
<text x="340" y="124" text-anchor="middle" font-size="13" class="diagram-muted">çevirir</text>
<text x="560" y="124" text-anchor="middle" font-size="13" class="diagram-muted">tek dosya, internetsiz</text>
</svg>
<figcaption>Gömülü bir SVG çizim. Renkler sınıfla verildiği için koyu temada da okunur.</figcaption>
</figure>

Video bloğu bir medya dosyası ister, bu yüzden burada yalnız yazımı gösteriliyor:

```markdown
:::video medya/tanitim.mp4 medya/tanitim-kapak.jpg
Videonun alt yazısı.
:::
```

Yerel Markdown görselinin yazımı aşağıdadır. Kütüphane çağıranı açıkça gömmeyi seçerse
PNG, JPEG, GIF ve WebP dosyaları sayfanın içine alınır; normal `uret` kullanımı
yolu olduğu gibi bırakır. Bu vitrin ek bir dosya gerektirmesin diye örnek kod olarak gösterilir:

```markdown
![Görselin açıklaması](medya/gorsel.png)
```

| Kavram | Kısa açıklama |
|---|---|
| Satır 1 | Tablolar Notion'daki gibi ince çizgilerle çizilir |
| Satır 2 | Başlık satırının zemini hafif gridir |

```python
def selam(isim: str) -> str:
    return f"Merhaba, {isim}!"
```

`text` etiketli blok düz metindir: uzun satır yana taşmaz, kutunun içinde alta iner. Bir modele verilen istem ya da bir günlük satırı böyle okunur:

```text
Bu satır kasıtlı olarak uzun yazıldı: kod bloklarında yana kaydırmak gerekir, ama text etiketli blokta satır kutunun sağ kenarına gelince bir alt satıra iner ve kutu sayfanın genişliğinde kalır.
```

Kod çitindeki blok adları uygulanmaz; bilinmeyen bir ad da burada örnek metin olarak gösterilebilir:

```text
:::ornek
Bu satırlar bir kapsayıcı açmaz.
:::
```

Dipnot da yazılabilir.[^1]

[^1]: Dipnotlar sayfanın en altında toplanır.

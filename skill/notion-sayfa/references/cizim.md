# Çizimler: sayfaya gömülü SVG

Görsel gereken yerde `<figure>` içine bir `<svg>` ve altına bir `<figcaption>` yazılır. Sayfa böylece tek dosya kalır. Canlı bir örnek [../templates/vitrin.md](../templates/vitrin.md) dosyasındadır.

## Kurallar

- **Boş satır yok.** `<figure>` ile `</figure>` arasında boş satır bırakılmaz. Markdown boş satırda HTML'i keser ve çizim bozulur.
- **Renkler sınıfla verilir.** Rengi elle yazma, aşağıdaki sınıfları kullan. Böylece çizim açık ve koyu temaya kendiliğinden uyar.

  | Sınıf | Ne |
  |---|---|
  | `diagram-text` | metin |
  | `diagram-muted` | soluk metin, küçük dolgu |
  | `diagram-line` | çizgi |
  | `diagram-box` | gri kutu |
  | `diagram-node` | mavi düğüm |
  | `diagram-level` | mavi dolum |
  | `diagram-accent` | turuncu dolgu |
  | `diagram-accent-line` | turuncu çizgi |

- **Boyut.** `viewBox` genişliği 680 civarında tutulur. Yazılar en az 13 birim olur. Telefonda yalnız çizim yatay kaydırılır: sağ kenarı solar, altında "← kaydır →" ipucu çıkar, alt yazı yerinde kalır. Bunu dönüştürücü yapar, çizime ek bir şey yazılmaz. Bir `<figure>` içinde tek `<svg>` bulunur.
- **Erişilebilirlik.** `<svg role="img" aria-label="…">` ile çizim bir cümleyle anlatılır.
- **Doğruluk.** Çizimdeki sayılar ve hesaplar doğru olmalıdır. Türkçe sayfada ondalık ayırıcı virgüldür: `0,5`, `%55`.

Sayfaya şöyle girer:

```markdown
<figure>
<svg viewBox="0 0 680 200" role="img" aria-label="Çizimi anlatan bir cümle.">
…
</svg>
<figcaption>Kısa, okura neye baktığını söyleyen bir cümle.</figcaption>
</figure>
```

## SVG'yi elle değil, küçük bir betikle üret

Birkaç satırdan uzun her çizimi geçici bir Python betiğiyle üret ve çıktısını sayfaya yapıştır. Böylece koordinatlar hesaplanır, düzeltme tek sayı değiştirmekle olur ve hesaplar tutarlı kalır.

- Konumları üstte adlı sabitler olarak tut (`ROW_TOP`, `BOX_X`, `FULL_BAR`).
- Veriyi (kelimeler, yüzdeler, değerler) tek bir listede tut. Çubuk genişlikleri, etiketler ve `aria-label` aynı veriden üretilsin.
- Rastgelelik gerekiyorsa tohumlu kullan: `random.Random(7)`. Her üretim aynı çizimi verir.
- Çıktıyı tek satır tek öğe olacak şekilde `"\n".join(...)` ile yaz. Boş satır oluşmaz.

İskelet:

```python
WIDTH, HEIGHT = 680, 200
OPTIONS = [("kedi", 55), ("köpek", 30), ("kuş", 15)]
BAR_X, FULL_BAR = 120, 300

def svg():
    parts = []
    for rank, (word, share) in enumerate(OPTIONS):
        y = 40 + rank * 28
        css = "diagram-accent" if rank == 0 else "diagram-level"
        parts.append(f'<text x="30" y="{y + 12}" font-size="14" class="diagram-text">{word}</text>')
        parts.append(f'<rect x="{BAR_X}" y="{y}" width="{FULL_BAR * share / 100:.1f}" height="14" rx="4" class="{css}"/>')
    label = "Adaylar: " + ", ".join(f"{w} yüzde {s}" for w, s in OPTIONS) + "."
    return "\n".join([f'<svg viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label="{label}">', *parts, "</svg>"])

print(svg())
```

## Görsel dil

Bir sayfadaki çizimler aynı dili konuşur, yenisi de ona uysun.

- **Seçilen, kazanan ya da yeni olan** turuncudur (`diagram-accent`, kalın yazı). Geri kalanı mavi (`diagram-node`, `diagram-level`) ya da gridir.
- Sütun başlıkları ve panel altı açıklamaları soluk ve küçüktür (`diagram-muted`, 13).
- İki panelli karşılaştırmada paneller arasına kesikli dikey çizgi konur (`stroke-dasharray="2 5"`), sağ panel `<g transform="translate(…)">` ile kaydırılır.
- Akış okları düz çizgi ve küçük üçgen uçtur. "Geri dönüş" gibi ikincil akışlar kesikli turuncu çizgidir.
- Adımlar numaralıysa soldaki küçük mavi dairelerle numaralanır.

## Sık hatalar ve kontrol

- **Etiket çizgiye biniyor.** Eğri yerine dik açılı yol kullan (`M … L … Q …` ile köşeleri yuvarla) ve etiketi yolun yatay parçasının üstüne koy.
- **Noktalama kelimeden kopuk.** Nokta ve virgül önceki kelimeye boşluksuz eklenir. Bir kelimeyi `<tspan>` ile vurgularken ayırıcıyı ona göre seç.
- **Metnin genişliği.** SVG yazı genişliğini ölçemez. Konum hesabında 15 punto için harf başına yaklaşık 7,4 birim say ve tarayıcıda gözle doğrula.
- **Hesap.** Yüzdeler 100'e tamamlanır, toplamlar ve çarpımlar gerçekten tutar.
- **İki tema.** Renk elle yazılmadıysa koyu temada da doğru görünür. Yine de tarayıcıda bir kez koyu temaya bak.
- **Telefon.** 680 genişlikte yazılar 13'ün altına inmesin. Telefonda çizim yatay kaydırılır.

---
name: turkish-writing
description: Write or adapt Turkish content with natural phrasing, consistent terminology, and correct spelling.
---

# Turkish writing guide

Use this guide when writing Turkish notes or adapting English content into Turkish.
Express meaning through natural Turkish sentence structure while preserving
technical details and the author's intent. Lead with what the reader needs, use
shared context in conversation, and give standalone documents enough context to
make sense on their own. Choose paragraphs, lists, and tables to fit the content.

The instructions are written in English. Turkish words and sentences below are
language examples, accompanied by English meanings where needed.

## How Turkish conveys meaning

Turkish carries much of its meaning through suffixes and the relationships between words.

- Turkish is agglutinative. The example `dosya-lar-ımız-dan` means “from our files.”
  When shortening text, preserve suffixes that express number, possession, case,
  person, or tense.
- Choose suffixes according to pronunciation and vowel harmony. Examples include
  `dosyada` (“in the file”), `bellekte` (“in memory”), and `dosyayı` (“the file,”
  as a definite object). Check foreign names and abbreviations individually.
- In neutral statements, the predicate usually comes last, as in `Uygulama dosyayı
  kaydeder.` (“The application saves the file.”). Word order is flexible, and
  changing it can affect emphasis and context. Place adjectives and modifying
  clauses before their nouns in most cases, as in `dün eklenen kayıt` (“the record
  added yesterday”).
- A subject understood from the verb's person suffix or context can be omitted.
  Both `Ben dosyayı kaydettim` and `Dosyayı kaydettim` mean “I saved the file,” but
  the second can be sufficient. Name the person or component when the actor would
  otherwise be unclear.
- Turkish has no direct equivalents for the English articles `a`, `an`, and `the`.
  Use `bir` when the number “one” or an indefinite meaning is needed. Preserve the
  distinction between `Kayıt seç` (“Select a record”) and `Kaydı seç` (“Select the record”).
- Turkish has no grammatical gender. The pronoun `o` (“he,” “she,” or “it”) carries
  no gender distinction. Use a person's name when a pronoun could refer to more
  than one person.

The discussion of agglutination, subjects, and word order draws on the
[introduction to A Student Grammar of Turkish](https://assets.cambridge.org/97805217/63462/excerpt/9780521763462_excerpt.pdf)
and the [Turkish UCCA study](https://www.cambridge.org/core/journals/natural-language-processing/article/building-a-turkish-ucca-dataset/38D65C2F4D10BBA002B092026C11F853).
The grammatical gender discussion draws on [this study of Turkish](https://dergipark.org.tr/en/pub/dtcfdergisi/article/1182365).
The writing preferences below are recommendations for this guide.

## Build natural sentences

Understand the meaning first, then construct the sentence in Turkish.

- Use Turkish participial and adverbial constructions where they fit. Examples
  include `başarısız olan istek` (“the request that failed”) and `dosyayı kaydederken`
  (“while saving the file”). When modifiers become hard to follow, split the
  sentence while preserving its reasons, conditions, and consequences.
- Prefer verbs when expressing actions. For example, `kontrol etmek` (“to check”)
  is more natural than `Kontrol işleminin gerçekleştirilmesi` (“the execution of
  the checking operation”). Prefer active voice when the actor matters and is
  known. Passive voice is appropriate when the actor is irrelevant or unknown,
  as in `Dosya bulunamadı.` (“The file could not be found.”).
- Reduce phrases such as `sahip olmak` (“to have”), `olmak` (“to be”), and
  `bir şekilde` (“in some way”) when they add no meaning. Use the aorist for
  general behavior and the present progressive for ongoing activity when those
  forms fit the note. Reserve the formal `-mektedir` ending for contexts that
  call for it.
- Include pronouns and possessives when they help the reader. Add `o` (“it”),
  `onun` (“its”), or `sizin` (“your”) according to Turkish context. If the referent
  of `bu` (“this”), `böyle` (“like this”), or `bunlar` (“these”) is unclear, name it.

These Turkish examples illustrate more direct wording. The English column gives
what each example means.

| Heavy or English-influenced phrasing | More natural Turkish | English meaning |
|---|---|---|
| `Araç üç farklı seçeneğe sahiptir.` | `Aracın üç seçeneği var.` | The tool has three options. |
| `Kullanıcı tarafından dosyanın silinmesi gerçekleştirildi.` | `Kullanıcı dosyayı sildi.` | The user deleted the file. |
| `Sunucu bir yanıt döndürme yeteneğine sahiptir.` | `Sunucu yanıt döndürebilir.` | The server can return a response. |
| `Bu, işlemin henüz tamamlanmadığı anlamına gelmektedir.` | `Bu ileti, işlemin henüz tamamlanmadığı anlamına gelir.` | This message means the operation is still incomplete. |

In the last example, the referent is known to be a message. Add a referent or
reason only when the available context supports it.

## Preserve meaning, emphasis, and tone

Keep the original degree of certainty and the relationships between ideas.

- Preserve the differences between `silebilir` (“can delete”), `siler` (“deletes”),
  and `silmeli` (“should delete”). Keep possibilities, predictions, recommendations,
  and obligations distinct. Preserve negation, conditions, and the scope of
  qualifiers such as `yalnızca` (“only”), `henüz` (“yet”), and `yeniden` (“again”).
- Pay attention to tense and the source of information. Depending on context,
  `silindi` (“was deleted”) and `silinmiş` (“apparently was deleted”) can express
  different ways of reporting or discovering an event. Identify the source when
  useful, as in `günlüğe göre` (“according to the log”). Match certainty to the evidence.
- Choose a consistent form of address. Turkish distinguishes informal singular
  `sen` and formal or plural `siz`, both corresponding to English “you.” Use the
  appropriate imperative consistently, such as `Dosyayı aç` or `Dosyayı açın`
  (“Open the file”). Explanatory prose can address the topic directly.
- Form headings as natural Turkish noun phrases, such as `Önbelleği temizleme`
  (“Clearing the cache”) and `Kurulum seçenekleri` (“Installation options”).
  Prefer sentence case while preserving proper nouns, the source heading's
  meaning, and any sequence numbers.

## Handle technical terms and code

Write explanations in Turkish and preserve expressions that must be executed or searched.

- Follow a supplied glossary. Otherwise, choose a familiar and accurate term,
  and use it consistently. Prefer established terminology that the reader understands.
- When helpful, give the Turkish and English terms together at first use, as in
  `gecikme (latency)`. Use the amount of terminology support the reader needs
  in later mentions.
- Preserve commands, options, code identifiers, file paths, and product names.
  Keep expressions such as `--output` and `get_user()` intact and explain them in Turkish.
- Place Turkish suffixes on surrounding words when that makes code easier to read.
  Examples include `` `config.json` dosyasında `` (“in the `config.json` file”) and
  `` `--output` seçeneğiyle `` (“with the `--output` option”). Preserve the case,
  punctuation, and numeric formatting within code.

## Check spelling

Preserve Turkish letters and distinguish separate particles from attached suffixes.

- Use the letters `ç, ğ, ı, ö, ş, ü`. Turkish case pairs include `i/İ` and `ı/I`.
  For example, the word for “communication” changes from `iletişim` to `İletişim`
  when capitalized.
- Write the additive particle `de/da` separately, as in `Önbellek de temizlendi.`
  (“The cache was also cleared.”). Attach the locative suffix, as in
  `Veri önbellekte.` (“The data is in the cache.”). Write the question particle
  separately and attach any following person suffix to it, as in `Kaydettin mi?`
  (“Did you save it?”) and `Kaydeder misin?` (“Would you save it?”). The conjunction
  `ki` is usually separate, as in `Anlaşılıyor ki` (“It appears that”), while the
  suffix `-ki` is attached, as in `dosyadaki` (“the one in the file”). Fixed forms
  such as `sanki` (“as if”) are exceptions. [TDK guidance on words and suffixes](https://yazim.tdk.gov.tr/content/03-bazi-kelime-ve-eklerin-yazilisi.html).
- Apply apostrophes according to the kind of name. Examples include `Ankara'da`
  (“in Ankara”), `TDK'nin` (“of TDK”), `Türk Dil Kurumunun` (“of the Turkish Language
  Association”), and `Türkçede` (“in Turkish”). Choose an abbreviation's suffix
  according to its pronunciation. [TDK guidance on apostrophes](https://tdk.gov.tr/icerik/yazim-kurallari/kesme-isareti/).

## Review the result

Read a standalone note from the perspective of a Turkish reader who has not seen
the conversation.

- Is it clear who did what, which words the suffixes modify, and what each pronoun refers to?
- Does the wording read naturally in Turkish, with clear actions and manageable modifiers?
- Did simplification preserve technical meaning, conditions, negation, and uncertainty?

## Earlier research

This guide adapts language and translation principles from the Oku project to
note writing. Its basis for natural Turkish, technical accuracy, and consistent
terminology comes from the [shared language rules](https://github.com/saadettinBerber/just_read/blob/60e5dfe8a448eb919abe58155fc3c22b011e9974/just_read/docs/language-style.md)
and [translation style rules](https://github.com/saadettinBerber/just_read/blob/60e5dfe8a448eb919abe58155fc3c22b011e9974/just_read/docs/translation-style.md).

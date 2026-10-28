# Coucou for Windows — o‘zbekcha versiya

Bu [Louis-CFM/coucou](https://github.com/Louis-CFM/coucou) loyihasidagi **Coucou for Windows**ning
eng so‘nggi versiyasi (**windows-v0.5.3**), to‘liq o‘zbek tiliga (lotin yozuvida) tarjima qilingan.

Coucou — ekraningiz tepasida yashaydigan kichkina do‘st Mochi. U Claude Code va boshqa AI
dasturlash agentlaringiz seanslarini kuzatib boradi: ruxsat so‘rovlarini tasdiqlash, savollarga
javob berish, chat, fayl tashlash, diktovka va boshqalar — qilayotgan ishingizni to‘xtatmasdan.

## Nimalar o‘zbekchaga o‘girildi

- Ilovaning barcha 1200+ matni: orolcha (ekran tepasidagi panel), sozlamalar oynasi, Coucou oynasi
  (Diktovka, Lug‘at, Shablonlar, Uslublar, Qayd yozuvchi, Statistika), xatolar, bildirishnomalar,
  tray menyusi, Mochi’ning kiyim shkafi va «Yangiliklar» kartasi — `windows/src/i18n/uz.json`.
- O‘rnatuvchi (setup.exe) — `windows/src-tauri/nsis/Uzbek.nsh`.
- Ilova har doim o‘zbek tilida ochiladi (kompyuteringiz tili qanday bo‘lishidan qat’i nazar).
  Boshqa tilni xohlasangiz: Sozlamalar → Umumiy → Til.
- Avtomatik yangilanish o‘chirilgan: aks holda ilova o‘zini asl inglizcha versiyaga yangilab,
  tarjimani yo‘qotib qo‘yardi.

## O‘rnatuvchini (exe) qanday olish mumkin

Har bir `push`da GitHub Actions Windows serverida o‘rnatuvchini yig‘adi:

1. Repozitoriyda **Actions** → **Windows o‘rnatuvchisini yig‘ish** ga kiring.
2. Eng so‘nggi yashil (✓) ishni oching.
3. Pastdagi **Artifacts** bo‘limidan `Coucou-Uzbek-0.5.3-setup` ni yuklab oling (zip ichida
   `Coucou-Uzbek-0.5.3-setup.exe`).
4. Zipdan chiqarib, `setup.exe` ni ishga tushiring. O‘rnatuvchi raqamli imzolanmagan, shuning
   uchun Windows SmartScreen ogohlantirish ko‘rsatadi: **Batafsil → Baribir ishga tushirish**
   ni bosing.

Qo‘lda yig‘ish (Windows’da, Node 22+ va Rust bilan):

```
cd windows
npm ci
npm run pack
```

Tayyor fayl: `windows/release/Coucou-Uzbek-0.5.3-setup.exe`.

## Litsenziya

Kod — MIT litsenziyasi ostida ([LICENSE](LICENSE)). «Coucou» va «Mochi» nomlari, Mochi qahramoni,
ikonka va tovushlar Louis Raillé’ga tegishli ([LICENSE-ASSETS.md](LICENSE-ASSETS.md)): bu
versiyani faqat shaxsiy foydalanish uchun yig‘ish mumkin, uni Coucou nomi, ikonkasi yoki
tovushlari bilan ommaga tarqatish uchun muallifning yozma ruxsati kerak.

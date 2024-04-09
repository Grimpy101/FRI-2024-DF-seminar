# FRI-2024-DF-seminar
Seminar pri predmetu Digitalna forenzika

Referenčni članek: https://dfrws.org/wp-content/uploads/2022/09/2022_APAC_paper-diaganalyzer_user_behavior_analysis_and_visualization_using_windows_diagnostics_logs.pdf

Načrt:
- program, ki analizira disk, na katerem je nameščen Windows OS
- prebere loge in baze za Windows 7 in naprej (če bo čas, lahko tudi še za XP ali podobno)
    - v članku so navedene lokacije, kjer je treba iskati
    - `Eventtrancript.db`
    - memory dump
    - `*.rbs` datoteke
- funkcije:
    - zazna dejavnosti naprav USB
    - zazna dejavnosti funkcije WiFi
    - analizira podatke brskanja po spletu
- UI:
    - grafična aplikacija ali kot v članku: ukazno orodje, ki izvozi HTML poročilo
- vizualizacije:
    - grafi za spletno dejavnost
    - časovnica dogodkov v nekem časovnem obdobju
    - vizualizacija lokacij glede na WiFi (z Google Maps)
- izvoz podatkov v datoteko (PDF? HTML? ODT?)
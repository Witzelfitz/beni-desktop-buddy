# Windows-Herausgebersignatur einrichten

Stand: 14. Oktober 2026. Noch kein Azure-Abonnement vorhanden. Es wurde keine kostenpflichtige Ressource angelegt. Die Installer-Pipeline baut derzeit ohne Authenticode-Herausgebersignatur. Unsere Ed25519-Downloadsignatur ersetzt diese nicht.

## 1. Azure-Abonnement

Für **Witzelfitz Consulting GmbH, St. Gallen, Schweiz** ist die öffentliche Organisationssignierung grundsätzlich verfügbar. Zuerst mit dem gewünschten Firmenkonto im [Azure-Portal](https://portal.azure.com/) anmelden und ein reguläres Azure-Abonnement einrichten. Rechtsname und Rechnungsdaten anhand des Handelsregisters prüfen. Keine Zahlungsdaten oder Passwörter in GitHub oder Chat kopieren.

**Kostenfreigabe erforderlich:** Artifact Signing Basic kostet laut [Microsoft](https://azure.microsoft.com/en-us/products/artifact-signing) USD 9.99 pro Monat für bis zu 5'000 Signaturen; zusätzliche Signaturen kosten USD 0.005. Steuern und Währungsumrechnung können hinzukommen. Vor der Ressourcenerstellung den im Portal angezeigten Preis bestätigen.

## 2. Signierkonto

Nach der ausdrücklichen Kostenfreigabe im Portal den Anbieter `Microsoft.CodeSigning` registrieren. Unter **Artifact Signing Accounts → Create** diese Werte prüfen:

| Feld | Vorschlag |
| --- | --- |
| Resource group | `beni-release-signing` |
| Account name | `witzelfitzbeni` (Verfügbarkeit prüfen) |
| Region | Switzerland North |
| Pricing tier | Basic |
| Endpoint | `https://swn.codesigning.azure.net` |

Die [Microsoft-Anleitung](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart) beschreibt die aktuellen Portalbezeichnungen. Account und Zertifikatsprofil sind getrennte Ressourcen.

## 3. Firmenprüfung

Im Signierkonto unter **Identity validations → Organization → New Identity → Public** die Firma prüfen lassen. Rechtsname, vollständige Geschäftsadresse, Schweizer Unternehmens-ID, Firmenwebsite und zwei erreichbare E-Mail-Adressen auf der eigenen Firmendomäne anhand der tatsächlichen Firmendaten eintragen. Die Prüfung erfolgt im Portal und benötigt gegebenenfalls Nachweise. Diese Identitätsprüfung kann die Build-Pipeline nicht übernehmen.

Erst nach erfolgreicher Prüfung ein Zertifikatsprofil **Public Trust** namens `beni-desktop` erstellen. **Private Trust** ist für öffentliche Downloads ungeeignet. Der bestätigte Firmenname wird Herausgeber des Zertifikats.

## 4. GitHub-Verknüpfung und Pipeline

Eine Entra-App mit föderierter GitHub-Identität einrichten. Die Berechtigung **Artifact Signing Certificate Profile Signer** auf das konkrete Profil begrenzen. Die Föderation auf das Release-Environment dieses Repositorys beschränken; keine Signierberechtigung für Pull Requests. GitHub benötigt `id-token: write`. Tenant-ID, Subscription-ID und Client-ID sind Konfiguration, keine privaten Schlüssel. Kein Client-Secret speichern.

Die [offizielle OIDC-Anleitung](https://github.com/Azure/artifact-signing-action/blob/main/docs/OIDC.md) dient als Grundlage für den nächsten Pipeline-PR, sobald die tatsächlichen Ressourcen und IDs vorhanden sind. Noch keine funktionslose Signierstufe aktivieren.

Vor der öffentlichen signierten Version sind **GUI, CLI, Deinstallationsprogramm und anschliessend der fertige Installer** mit SHA-256 und RFC-3161-Zeitstempel zu signieren. NSIS muss das Deinstallationsprogramm vor dem Einbetten signieren. Danach Signaturen über die Windows-Vertrauenskette prüfen, den fertigen Installer nativ abnehmen und erst diese endgültigen Bytes mit Ed25519 signieren. Ein bestehender Release wird nicht nachträglich verändert.

Auch ein korrekt signierter neuer Download kann anfangs eine [SmartScreen-Reputationswarnung](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) auslösen. Signierung garantiert keine sofortige Reputation.

## Abnahme

- **PASS:** Preis und Schweizer Organisationsverfügbarkeit anhand offizieller Quellen geprüft; Anleitung vorbereitet.
- **NOT_RUN:** Abonnement, Ressourcen, Firmenprüfung, OIDC-Anbindung und Authenticode-Signierung. Dafür fehlen derzeit Abonnement und freigegebene Ressourcen.
- **Separat:** Windows-Installer und Linux-AppImage werden ohne diesen externen Schritt gebaut und auf nativen Runnern abgenommen. Öffentliche Verfügbarkeit ist im Release und auf der Downloadseite zu bestätigen.

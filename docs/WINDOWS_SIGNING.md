# Windows-Herausgebersignatur einrichten

Stand: 14. Oktober 2026. Kosten für Artifact Signing Basic ausdrücklich freigegeben. Aktives Azure-Abonnement mit Besitzerrolle und `Microsoft.CodeSigning`-Registrierung durch den Nutzer bestätigt. Konto `witzelfitzbeni` in Ressourcengruppe `beni-release-signing`, Switzerland North, Basic und Konto-URI im Portal-Screenshot geprüft. Die Identity-Verifier-Rolle wurde dem eigenen Benutzerkonto zugewiesen. **Nächster Schritt: öffentliche Firmenprüfung.** Noch kein Public-Trust-Profil, keine OIDC-Anbindung und keine Authenticode-Signierung. Die Ed25519-Downloadsignatur ersetzt diese nicht.

## 1. Azure-Abonnement

Für **Witzelfitz Consulting GmbH, St. Gallen, Schweiz** ist die öffentliche Organisationssignierung grundsätzlich verfügbar. Zuerst mit dem gewünschten Firmenkonto im [Azure-Portal](https://portal.azure.com/) anmelden und ein reguläres Azure-Abonnement einrichten. Rechtsname und Rechnungsdaten anhand des Handelsregisters prüfen. Keine Zahlungsdaten oder Passwörter in GitHub oder Chat kopieren.

**Kostenfreigabe vorhanden:** Artifact Signing Basic kostet laut [Microsoft](https://azure.microsoft.com/en-us/products/artifact-signing) USD 9.99 pro Monat für bis zu 5'000 Signaturen; zusätzliche Signaturen kosten USD 0.005. Steuern und Währungsumrechnung können hinzukommen. Die Freigabe gilt für Basic. [Kostenlose, Probe- und gesponserte Abonnements sind ungeeignet](https://learn.microsoft.com/en-us/azure/artifact-signing/faq#can-i-use-artifact-signing-with-a-free-trial-or-sponsored-azure-subscription).

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

- **PASS:** Preis und Schweizer Organisationsverfügbarkeit anhand offizieller Quellen geprüft; Basic-Kosten freigegeben. Aktives Abonnement/Provider-Registrierung durch den Nutzer bestätigt; Basic-Konto mit Region und URI im Portal-Screenshot geprüft; eigene Identity-Verifier-Rolle zugewiesen.
- **PASS Windows-Paket:** Nativer Installer und eigenes Fenster auf dem Server-2025-Runner mit isoliertem Mesa-Softwaretreiber abgenommen, [Run 38537805266](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38537805266). Diese unsigned Ausgabe ist nicht als öffentlicher Download angeboten.
- **NOT_RUN:** Öffentliche Firmenprüfung, Public-Trust-Profil, OIDC, Authenticode und öffentliche Windows-Veröffentlichung. Bisher wird der Nutzer durch die Portal-Einrichtung geführt; keine API-Anmeldung für Azure eingerichtet. Identitätsnachweise und allfällige weitere Microsoft-Prüfung bleiben beim Nutzer/Microsoft.
- **Separat abgeschlossen:** Linux-AppImage und beide Mac-DMGs sind als 0.1.1 öffentlich verfügbar und auf der Downloadseite geprüft; siehe [Installerabnahme](PORTABLE_INSTALLERS.md).

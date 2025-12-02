//! Interactive CLI mode
//!
//! Main interactive menu system using dialoguer.

use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::application::workflow::Workflow;
use std::io;

/// Check if an error is a user cancellation (Ctrl+C, ESC, etc.)
/// Returns true if the error should be treated as a cancellation
fn is_user_cancellation(error: &anyhow::Error) -> bool {
    if let Some(io_error) = error.downcast_ref::<io::Error>() {
        matches!(
            io_error.kind(),
            io::ErrorKind::Interrupted | io::ErrorKind::UnexpectedEof
        )
    } else {
        // Check error message for common cancellation patterns
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("interrupted") || error_msg.contains("cancelled")
    }
}

/// Main menu options
#[derive(Debug, Clone, Copy)]
enum MainMenuOption {
    AddAccount,
    ScanAccount,
    CleanAccount,
    ListAccounts,
    Exit,
}

impl std::fmt::Display for MainMenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuOption::AddAccount => write!(f, "➕ Ajouter un compte Gmail"),
            MainMenuOption::ScanAccount => write!(f, "🔍 Scanner une boîte mail"),
            MainMenuOption::CleanAccount => write!(f, "🧹 Nettoyer une boîte mail"),
            MainMenuOption::ListAccounts => write!(f, "📋 Lister les comptes"),
            MainMenuOption::Exit => write!(f, "🚪 Quitter"),
        }
    }
}

/// Run the interactive CLI
pub async fn run_interactive(workflow: Workflow) -> Result<()> {
    println!("\n🔹 UnsubMail - Nettoyez votre Gmail\n");

    loop {
        let options = vec![
            MainMenuOption::AddAccount,
            MainMenuOption::ScanAccount,
            MainMenuOption::CleanAccount,
            MainMenuOption::ListAccounts,
            MainMenuOption::Exit,
        ];

        let selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Que voulez-vous faire ?")
            .items(&options)
            .default(0)
            .interact()
        {
            Ok(sel) => sel,
            Err(e) => {
                // If user cancelled main menu (Ctrl+C), exit gracefully
                let err = anyhow::Error::from(e);
                if is_user_cancellation(&err) {
                    println!("\n👋 À bientôt !\n");
                    return Ok(());
                }
                return Err(err);
            }
        };

        match options[selection] {
            MainMenuOption::AddAccount => {
                if let Err(e) = handle_add_account(&workflow).await {
                    if is_user_cancellation(&e) {
                        // User cancelled, return to menu
                        continue;
                    }
                    // Other errors should still propagate
                    return Err(e);
                }
            }
            MainMenuOption::ScanAccount => {
                if let Err(e) = handle_scan_account(&workflow).await {
                    if is_user_cancellation(&e) {
                        // User cancelled, return to menu
                        continue;
                    }
                    // Other errors should still propagate
                    return Err(e);
                }
            }
            MainMenuOption::CleanAccount => {
                if let Err(e) = handle_clean_account(&workflow).await {
                    if is_user_cancellation(&e) {
                        // User cancelled, return to menu
                        continue;
                    }
                    // Other errors should still propagate
                    return Err(e);
                }
            }
            MainMenuOption::ListAccounts => {
                if let Err(e) = handle_list_accounts(&workflow).await {
                    if is_user_cancellation(&e) {
                        // User cancelled, return to menu
                        continue;
                    }
                    // Other errors should still propagate
                    return Err(e);
                }
            }
            MainMenuOption::Exit => {
                println!("\n👋 À bientôt !\n");
                break;
            }
        }

        println!(); // Empty line between operations
    }

    Ok(())
}

/// Handle add account flow
async fn handle_add_account(workflow: &Workflow) -> Result<()> {
    use dialoguer::Input;

    println!("\n📧 Ajouter un nouveau compte Gmail\n");

    let email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Adresse email Gmail")
        .interact_text()
        .map_err(|e| anyhow::Error::from(e))?;

    println!("\n🔐 Lancement du processus d'authentification...\n");

    match workflow.add_account_interactive(&email).await {
        Ok(()) => {
            println!("\n✅ Compte {} ajouté avec succès !", email);
        }
        Err(e) => {
            eprintln!("\n❌ Erreur lors de l'ajout du compte: {}", e);
        }
    }

    Ok(())
}

/// Handle scan account flow
async fn handle_scan_account(workflow: &Workflow) -> Result<()> {
    let email = select_account(workflow, "Quel compte voulez-vous scanner ?").await?;

    if let Some(email) = email {
        println!("\n🔍 Scan de {}...\n", email);

        match workflow.scan_account(&email).await {
            Ok(senders) => {
                println!("\n✅ Scan terminé ! {} expéditeurs uniques trouvés\n", senders.len());
                println!("📊 Top 10 candidats newsletter :\n");

                for (i, sender) in senders.iter().take(10).enumerate() {
                    let unsub_str = if sender.has_one_click {
                        "✓ one-click"
                    } else if sender.has_unsubscribe {
                        "⚠ manuel"
                    } else {
                        "✗ aucun"
                    };

                    println!("  {}. {} ({} msgs) [{}] - score: {:.2}",
                        i + 1,
                        sender.display_name.as_ref().unwrap_or(&sender.email),
                        sender.message_count,
                        unsub_str,
                        sender.score
                    );
                }
            }
            Err(e) => {
                eprintln!("\n❌ Erreur lors du scan: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle clean account flow
async fn handle_clean_account(workflow: &Workflow) -> Result<()> {
    let email = select_account(workflow, "Quel compte voulez-vous nettoyer ?").await?;

    if let Some(email) = email {
        println!("\n🧹 Nettoyage de {}...\n", email);
        println!("🔍 Scan en cours...\n");

        match workflow.scan_account(&email).await {
            Ok(senders) => {
                if senders.is_empty() {
                    println!("ℹ Aucun expéditeur trouvé à nettoyer.");
                    return Ok(());
                }

                println!("✓ {} expéditeurs trouvés\n", senders.len());

                // Select senders to clean
                let selections = match crate::cli::select::select_senders(&senders) {
                    Ok(selections) => selections,
                    Err(e) => {
                        // If user cancelled selection, return to menu
                        if is_user_cancellation(&e) {
                            return Err(e);
                        }
                        // Other errors should be handled
                        eprintln!("\n❌ Erreur lors de la sélection: {}", e);
                        return Ok(());
                    }
                };

                if selections.is_empty() {
                    println!("ℹ Aucun expéditeur sélectionné.");
                    return Ok(());
                }

                println!("\n🚀 Nettoyage de {} expéditeurs...\n", selections.len());

                match workflow.cleanup_account(&email, selections, &senders).await {
                    Ok(results) => {
                        // Summary
                        println!("\n📊 === Résumé du nettoyage ===\n");
                        let mut total_deleted = 0;
                        let mut total_unsubscribed = 0;
                        let mut total_blocked = 0;

                        for result in results {
                            total_deleted += result.messages_deleted;
                            if result.unsubscribed {
                                total_unsubscribed += 1;
                            }
                            if result.blocked {
                                total_blocked += 1;
                            }
                        }

                        println!("✓ Désabonnements: {}", total_unsubscribed);
                        println!("✓ Bloqués: {}", total_blocked);
                        println!("✓ Messages supprimés: {}", total_deleted);
                        println!();
                    }
                    Err(e) => {
                        eprintln!("\n❌ Erreur lors du nettoyage: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ Erreur lors du scan: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle list accounts
async fn handle_list_accounts(workflow: &Workflow) -> Result<()> {
    println!("\n📋 Comptes configurés:\n");

    match workflow.list_accounts().await {
        Ok(accounts) => {
            if accounts.is_empty() {
                println!("ℹ Aucun compte configuré. Ajoutez-en un avec l'option 'Ajouter un compte'.\n");
            } else {
                for account in accounts {
                    println!("  • {} (ajouté le: {})",
                        account.email,
                        account.added_at.format("%d/%m/%Y à %H:%M")
                    );
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("\n❌ Erreur lors de la récupération des comptes: {}", e);
        }
    }

    Ok(())
}

/// Helper to select an account from the list
async fn select_account(workflow: &Workflow, prompt: &str) -> Result<Option<String>> {
    let accounts = workflow.list_accounts().await?;

    if accounts.is_empty() {
        println!("\n⚠ Aucun compte configuré. Ajoutez d'abord un compte.\n");
        return Ok(None);
    }

    let emails: Vec<String> = accounts.iter().map(|a| a.email.clone()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&emails)
        .default(0)
        .interact()
        .map_err(|e| anyhow::Error::from(e))?;

    Ok(Some(emails[selection].clone()))
}

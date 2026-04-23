import * as vscode from 'vscode';

/** Welcome message shown to first-time users. */
export interface WelcomeMessage {
  title: string;
  body: string;
  emoji: string;
}

/** Quick example prompt buttons for the welcome card. */
export interface ExamplePrompt {
  id: string;
  label: string;
  icon: string;
  text: string;
}

/** Single step in the interactive onboarding tour. */
export interface TourStep {
  id: string;
  target: string;
  message: string;
}

/** Onboarding progress tracking. */
export interface OnboardingProgress {
  stepIndex: number;
  totalSteps: number;
  completed: boolean;
}

/** OnboardingManager — Guides new users through first-time setup.
 *
 *  Detects first-time users via globalState, shows a friendly welcome
 *  message with quick example buttons, and offers a dismissible tour.
 *  Returning users see nothing. Preferences are persisted across
 *  extension host restarts.
 *
 *  Key behaviors:
 *  - isFirstTimeUser: checks globalState for onboarded/dismissed flags
 *  - getWelcomeMessage: friendly greeting with emoji
 *  - getQuickExamples: 4 one-click starter prompts
 *  - getTourSteps: 4-step guided walkthrough
 *  - dismiss: permanently hides onboarding
 *  - progress tracking: remembers which tour steps were completed
 *
 *  Storage schema (globalState):
 *  - hajimi.onboarded: boolean — user has completed onboarding
 *  - hajimi.onboarding.dismissed: boolean — user dismissed onboarding
 *  - hajimi.onboarding.progress: OnboardingProgress — tour step tracking
 */
export class OnboardingManager {
  private static readonly STORAGE_KEY = 'hajimi.onboarded';
  private static readonly DISMISS_KEY = 'hajimi.onboarding.dismissed';
  private static readonly PROGRESS_KEY = 'hajimi.onboarding.progress';

  constructor(private context: vscode.ExtensionContext) {}

  /** Check if the user has never seen onboarding before. */
  public isFirstTimeUser(): boolean {
    const onboarded = this.context.globalState.get<boolean>(OnboardingManager.STORAGE_KEY, false);
    const dismissed = this.context.globalState.get<boolean>(OnboardingManager.DISMISS_KEY, false);
    return !onboarded && !dismissed;
  }

  /** Check if onboarding was permanently dismissed. */
  public isDismissed(): boolean {
    return this.context.globalState.get<boolean>(OnboardingManager.DISMISS_KEY, false);
  }

  /** Mark onboarding as completed (user has seen it). */
  public async markCompleted(): Promise<void> {
    await this.context.globalState.update(OnboardingManager.STORAGE_KEY, true);
  }

  /** Permanently dismiss onboarding for this user. */
  public async dismiss(): Promise<void> {
    await this.context.globalState.update(OnboardingManager.DISMISS_KEY, true);
    await this.context.globalState.update(OnboardingManager.PROGRESS_KEY, { stepIndex: 0, totalSteps: 0, completed: true });
  }

  /** Reset onboarding state (for testing or "show again" feature). */
  public async reset(): Promise<void> {
    await this.context.globalState.update(OnboardingManager.STORAGE_KEY, false);
    await this.context.globalState.update(OnboardingManager.DISMISS_KEY, false);
    await this.context.globalState.update(OnboardingManager.PROGRESS_KEY, { stepIndex: 0, totalSteps: 0, completed: false });
  }

  /** Get current onboarding progress. */
  public getProgress(): OnboardingProgress {
    return this.context.globalState.get<OnboardingProgress>(OnboardingManager.PROGRESS_KEY, { stepIndex: 0, totalSteps: 4, completed: false });
  }

  /** Advance to the next tour step and persist progress. */
  public async advanceStep(): Promise<void> {
    const progress = this.getProgress();
    const next = Math.min(progress.stepIndex + 1, progress.totalSteps);
    await this.context.globalState.update(OnboardingManager.PROGRESS_KEY, { ...progress, stepIndex: next, completed: next >= progress.totalSteps });
  }

  /** Friendly welcome message for first-time users. */
  public getWelcomeMessage(): WelcomeMessage {
    return {
      title: 'Welcome to Hajimi',
      body: 'Your AI pair programmer. Type a message or pick a quick example below to get started. Use @filename to reference files and #folder for directories.',
      emoji: '🤖',
    };
  }

  /** Quick example prompts shown as buttons in the welcome card. */
  public getQuickExamples(): ExamplePrompt[] {
    return [
      { id: 'ex-build', label: 'Build project', icon: '🔨', text: '/build' },
      { id: 'ex-explain', label: 'Explain selected code', icon: '🔍', text: 'Explain the selected code' },
      { id: 'ex-fix', label: 'Fix a bug', icon: '🐛', text: 'Fix the bug in the current file' },
      { id: 'ex-test', label: 'Run tests', icon: '🧪', text: '/test' },
    ];
  }

  /** Interactive tour steps highlighting key UI areas. */
  public getTourSteps(): TourStep[] {
    return [
      { id: 'tour-chat', target: 'Chat panel', message: 'Type messages here. Use /commands and @files for smart context.' },
      { id: 'tour-trace', target: 'Thinking Trace', message: 'Watch the AI think through 7 steps in real-time.' },
      { id: 'tour-diff', target: 'Diff Preview', message: 'Review edits before applying. Accept, Reject, or Explain.' },
      { id: 'tour-tools', target: 'Quick Tools', message: 'One-click access to Build, Test, Git, and more.' },
    ];
  }

  /** Show a transient info message for tour guidance. */
  public showTourStep(step: TourStep): void {
    void vscode.window.showInformationMessage(`[${step.target}] ${step.message}`);
  }

  /** Check if the tour is fully completed. */
  public isTourCompleted(): boolean {
    return this.getProgress().completed;
  }

  /** Returns true if onboarding should be shown right now. */
  public shouldShow(): boolean {
    return this.isFirstTimeUser() && !this.isDismissed();
  }
}

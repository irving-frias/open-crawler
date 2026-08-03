import { invoke } from '@tauri-apps/api/core';
import type { Project, ProjectStats } from './types';

export function listProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

export function getProject(id: string): Promise<Project> {
  return invoke<Project>('get_project', { id });
}

export function createProject(name: string): Promise<Project> {
  return invoke<Project>('create_project', { request: { name } });
}

export function renameProject(id: string, name: string): Promise<void> {
  return invoke<void>('rename_project', { request: { id, name } });
}

export function deleteProject(id: string): Promise<void> {
  return invoke<void>('delete_project', { id });
}

export function getProjectStats(projectId: string): Promise<ProjectStats> {
  return invoke<ProjectStats>('get_project_stats', { projectId });
}

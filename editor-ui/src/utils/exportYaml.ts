/**
 * Utility functions for exporting semantic models to YAML files.
 * 
 * Requirements: 6.1 - THE Editor SHALL provide an export button that downloads 
 * the current model as a YAML file
 */

import yaml from 'js-yaml';
import type { SemanticModel } from '../types';

/**
 * Converts a SemanticModel to a YAML string.
 * 
 * @param model - The semantic model to convert
 * @returns YAML string representation of the model
 */
export function modelToYaml(model: SemanticModel): string {
  // Use js-yaml to serialize the model with nice formatting
  return yaml.dump(model, {
    indent: 2,
    lineWidth: 120,
    noRefs: true,
    sortKeys: false,
    quotingType: '"',
    forceQuotes: false,
  });
}

/**
 * Triggers a browser download of the given content as a file.
 * 
 * @param content - The file content to download
 * @param filename - The name of the file to download
 * @param mimeType - The MIME type of the file
 */
export function downloadFile(content: string, filename: string, mimeType: string = 'text/yaml'): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  
  // Append to body, click, and remove
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  
  // Clean up the URL object
  URL.revokeObjectURL(url);
}

/**
 * Exports a semantic model as a YAML file download.
 * 
 * @param model - The semantic model to export
 * @param filename - Optional custom filename (defaults to model name)
 */
export function exportModelAsYaml(model: SemanticModel, filename?: string): void {
  const yamlContent = modelToYaml(model);
  
  // Generate filename from model name if not provided
  const sanitizedName = model.name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '') || 'semantic-model';
  
  const finalFilename = filename || `${sanitizedName}.yaml`;
  
  downloadFile(yamlContent, finalFilename);
}

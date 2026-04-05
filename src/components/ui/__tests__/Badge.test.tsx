import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Badge from '../Badge';

describe('Badge', () => {
  it('renders children text', () => {
    render(<Badge>New</Badge>);
    expect(screen.getByText('New')).toBeInTheDocument();
  });

  it('applies default variant classes', () => {
    render(<Badge>Default</Badge>);
    const el = screen.getByText('Default');
    expect(el.className).toContain('bg-gray-100');
  });

  it('applies danger variant classes', () => {
    render(<Badge variant="danger">Error</Badge>);
    const el = screen.getByText('Error');
    expect(el.className).toContain('bg-red-100');
  });

  it('applies success variant classes', () => {
    render(<Badge variant="success">OK</Badge>);
    const el = screen.getByText('OK');
    expect(el.className).toContain('bg-green-100');
  });
});

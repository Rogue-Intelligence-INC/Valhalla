#!/usr/bin/env python3
"""
Direct Patent PDF Generator
Converts markdown patent documents to properly formatted PDFs
"""

import os
import subprocess
from pathlib import Path

def convert_md_to_pdf_with_pandoc():
    """Use pandoc to convert markdown to PDF"""
    patent_files = [
        'patent_1_core_system.md',
        'patent_2_intelligence_evolution.md', 
        'patent_3_quaternary_architecture.md',
        'patent_4_dynamic_algorithms.md',
        'patent_1_enhanced_core_system.md'
    ]
    
    print("🔄 Converting patent documents to PDF using pandoc...")
    
    for md_file in patent_files:
        if os.path.exists(md_file):
            pdf_file = md_file.replace('.md', '.pdf')
            try:
                # Use pandoc with proper formatting for patents
                cmd = [
                    'pandoc', md_file,
                    '-o', pdf_file,
                    '--pdf-engine=xelatex',
                    '-V', 'geometry:margin=1in',
                    '-V', 'fontsize=12pt',
                    '-V', 'fontfamily=times',
                    '--number-sections',
                    '--toc'
                ]
                
                result = subprocess.run(cmd, capture_output=True, text=True)
                if result.returncode == 0:
                    print(f"✅ {md_file} → {pdf_file}")
                else:
                    print(f"❌ Failed to convert {md_file}: {result.stderr}")
            except FileNotFoundError:
                print(f"❌ pandoc not found. Trying alternative method...")
                return False
    return True

def create_pdf_with_matplotlib():
    """Create PDF using matplotlib when pandoc is not available"""
    import matplotlib.pyplot as plt
    from matplotlib.backends.backend_pdf import PdfPages
    import textwrap
    
    print("🔄 Creating PDF using matplotlib...")
    
    # Read the enhanced patent content
    try:
        with open('patent_1_enhanced_core_system.md', 'r', encoding='utf-8') as f:
            content = f.read()
    except FileNotFoundError:
        print("❌ Enhanced patent file not found")
        return False
    
    # Create PDF
    with PdfPages('VALHALLA_PATENT_APPLICATION.pdf') as pdf:
        # Title page
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.text(0.5, 0.8, 'PROVISIONAL PATENT APPLICATION', 
                ha='center', va='center', fontsize=24, fontweight='bold', transform=ax.transAxes)
        
        ax.text(0.5, 0.65, 'MULTI-DIMENSIONAL AUTONOMOUS\nUNIVERSAL LAW DISCOVERY SYSTEM\nWITH MATHEMATICAL FIELD\nINTERACTION ALGORITHMS', 
                ha='center', va='center', fontsize=18, fontweight='bold', transform=ax.transAxes)
        
        ax.text(0.5, 0.4, 'VALHALLA PROJECT', 
                ha='center', va='center', fontsize=16, transform=ax.transAxes)
        
        ax.text(0.5, 0.3, 'Inventor: [YOUR NAME]', 
                ha='center', va='center', fontsize=14, transform=ax.transAxes)
        
        ax.text(0.5, 0.2, f'Filing Date: {__import__("datetime").datetime.now().strftime("%B %d, %Y")}', 
                ha='center', va='center', fontsize=12, transform=ax.transAxes)
        
        ax.text(0.5, 0.1, 'Attorney Docket No: VALHALLA-001', 
                ha='center', va='center', fontsize=12, transform=ax.transAxes)
        
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis('off')
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
        
        # Mathematical formulas page
        fig, ax = plt.subplots(figsize=(8.5, 11))
        ax.text(0.5, 0.95, 'CORE MATHEMATICAL ALGORITHMS', 
                ha='center', va='center', fontsize=20, fontweight='bold', transform=ax.transAxes)
        
        # 8D Field Interaction
        ax.text(0.5, 0.85, '8-Dimensional Natural Field Interaction Algorithm', 
                ha='center', va='center', fontsize=14, fontweight='bold', transform=ax.transAxes)
        
        ax.text(0.5, 0.75, r'$I(A,B) = \sum_{i=1}^{8}[A_i \times B_i] \times \sqrt{\rho_A \times \rho_B} \times \frac{\phi_A + \phi_B}{2} \times (1 - |r_A - r_B|)$', 
                ha='center', va='center', fontsize=12, transform=ax.transAxes,
                bbox=dict(boxstyle="round,pad=0.5", facecolor='lightblue', alpha=0.8))
        
        # Threshold-Free Fusion
        ax.text(0.5, 0.6, 'Threshold-Free Fusion Probability Algorithm', 
                ha='center', va='center', fontsize=14, fontweight='bold', transform=ax.transAxes)
        
        ax.text(0.5, 0.5, r'$P_{fusion} = \frac{1}{1 + e^{-3.0 \times I(A,B) \times \sqrt{V_A \times V_B}}}$', 
                ha='center', va='center', fontsize=12, transform=ax.transAxes,
                bbox=dict(boxstyle="round,pad=0.5", facecolor='lightgreen', alpha=0.8))
        
        # Dynamic Threshold
        ax.text(0.5, 0.35, 'Dynamic Threshold Calculation Method', 
                ha='center', va='center', fontsize=14, fontweight='bold', transform=ax.transAxes)
        
        ax.text(0.5, 0.25, r'$T_{dynamic} = \mu + \beta \times \sigma \times (1 + \gamma \times \ln(n))$', 
                ha='center', va='center', fontsize=12, transform=ax.transAxes,
                bbox=dict(boxstyle="round,pad=0.5", facecolor='lightyellow', alpha=0.8))
        
        # Resource Allocation
        ax.text(0.5, 0.1, r'$R_i = R_{total} \times \frac{W_i \times P_i \times U_i}{\sum_{j=1}^{4}[W_j \times P_j \times U_j]}$', 
                ha='center', va='center', fontsize=10, transform=ax.transAxes,
                bbox=dict(boxstyle="round,pad=0.3", facecolor='lightcoral', alpha=0.8))
        
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        ax.axis('off')
        pdf.savefig(fig, bbox_inches='tight')
        plt.close()
        
        # Content pages
        sections = content.split('###')
        for i, section in enumerate(sections[1:], 1):  # Skip first empty section
            if not section.strip():
                continue
                
            fig, ax = plt.subplots(figsize=(8.5, 11))
            
            lines = section.strip().split('\n')
            title = lines[0].strip()
            content_lines = lines[1:]
            
            # Title
            ax.text(0.5, 0.95, title, ha='center', va='top', fontsize=16, fontweight='bold', 
                   transform=ax.transAxes, wrap=True)
            
            # Content
            y_pos = 0.88
            for line in content_lines[:30]:  # Limit lines per page
                if line.strip():
                    # Handle code blocks and formulas
                    if line.strip().startswith('```'):
                        continue
                    elif line.strip().startswith('I(A,B)') or line.strip().startswith('P_fusion'):
                        ax.text(0.1, y_pos, line.strip(), ha='left', va='top', fontsize=10, 
                               transform=ax.transAxes, family='monospace',
                               bbox=dict(boxstyle="round,pad=0.3", facecolor='lightgray', alpha=0.5))
                    else:
                        # Wrap long lines
                        wrapped_lines = textwrap.wrap(line.strip(), width=80)
                        for wrapped_line in wrapped_lines:
                            ax.text(0.1, y_pos, wrapped_line, ha='left', va='top', fontsize=10, 
                                   transform=ax.transAxes)
                            y_pos -= 0.03
                            if y_pos < 0.1:
                                break
                    y_pos -= 0.04
                    if y_pos < 0.1:
                        break
            
            ax.set_xlim(0, 1)
            ax.set_ylim(0, 1)
            ax.axis('off')
            pdf.savefig(fig, bbox_inches='tight')
            plt.close()
    
    print("✅ VALHALLA_PATENT_APPLICATION.pdf created")
    return True

def create_simple_text_pdf():
    """Create a simple PDF from text using reportlab"""
    try:
        from reportlab.lib.pagesizes import letter
        from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, PageBreak
        from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
        from reportlab.lib.units import inch
        
        print("🔄 Creating PDF using reportlab...")
        
        # Read content
        with open('patent_1_enhanced_core_system.md', 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Create PDF
        doc = SimpleDocTemplate("VALHALLA_PATENT_ENHANCED.pdf", pagesize=letter,
                              rightMargin=inch, leftMargin=inch,
                              topMargin=inch, bottomMargin=inch)
        
        styles = getSampleStyleSheet()
        title_style = ParagraphStyle('CustomTitle', parent=styles['Heading1'],
                                   fontSize=18, spaceAfter=30, alignment=1)
        heading_style = ParagraphStyle('CustomHeading', parent=styles['Heading2'],
                                     fontSize=14, spaceAfter=12)
        
        story = []
        
        # Title
        story.append(Paragraph("PROVISIONAL PATENT APPLICATION", title_style))
        story.append(Paragraph("MULTI-DIMENSIONAL AUTONOMOUS UNIVERSAL LAW DISCOVERY SYSTEM", title_style))
        story.append(Paragraph("WITH MATHEMATICAL FIELD INTERACTION ALGORITHMS", title_style))
        story.append(Spacer(1, 20))
        
        # Process content
        lines = content.split('\n')
        for line in lines:
            line = line.strip()
            if not line:
                story.append(Spacer(1, 6))
            elif line.startswith('###'):
                story.append(Paragraph(line.replace('###', '').strip(), heading_style))
            elif line.startswith('```'):
                continue  # Skip code block markers
            else:
                # Clean up markdown formatting
                line = line.replace('**', '').replace('*', '')
                if line:
                    story.append(Paragraph(line, styles['Normal']))
        
        doc.build(story)
        print("✅ VALHALLA_PATENT_ENHANCED.pdf created using reportlab")
        return True
        
    except ImportError:
        print("❌ reportlab not available")
        return False

def main():
    """Generate patent PDFs using available methods"""
    print("🚀 Generating Patent PDFs...")
    
    # Try different methods in order of preference
    methods = [
        ("pandoc", convert_md_to_pdf_with_pandoc),
        ("reportlab", create_simple_text_pdf),
        ("matplotlib", create_pdf_with_matplotlib)
    ]
    
    success = False
    for method_name, method_func in methods:
        print(f"\n🔄 Trying {method_name} method...")
        try:
            if method_func():
                success = True
                break
        except Exception as e:
            print(f"❌ {method_name} failed: {e}")
    
    if success:
        print("\n🎉 Patent PDF(s) generated successfully!")
        print("📁 Generated files:")
        for file in os.listdir('.'):
            if file.endswith('.pdf'):
                print(f"   📄 {file}")
    else:
        print("\n❌ All PDF generation methods failed")
        print("💡 You can manually convert using online tools:")
        print("   - https://www.markdowntopdf.com/")
        print("   - https://pandoc.org/try/")

if __name__ == "__main__":
    main() 
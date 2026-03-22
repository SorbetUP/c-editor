#!/bin/bash

# Script pour générer un corpus de test de grande taille
# Usage: ./generate_large_corpus.sh <nombre_de_notes>

NOTES_COUNT=${1:-1000}
BASE_DIR="large_test_vault"

echo "🚀 Génération d'un corpus de test de $NOTES_COUNT notes..."

# Nettoyer le répertoire existant
rm -rf "$BASE_DIR"
mkdir -p "$BASE_DIR"

# Créer la structure de dossiers
mkdir -p "$BASE_DIR/Notes/Projects"
mkdir -p "$BASE_DIR/Notes/Research" 
mkdir -p "$BASE_DIR/Notes/Meetings"
mkdir -p "$BASE_DIR/Notes/Archive"
mkdir -p "$BASE_DIR/Notes/Daily"
mkdir -p "$BASE_DIR/Templates"
mkdir -p "$BASE_DIR/Documentation"
mkdir -p "$BASE_DIR/Ideas"

# Listes de mots pour génération de contenu varié
declare -a TOPICS=("artificial intelligence" "machine learning" "neural networks" "deep learning" "natural language processing" "computer vision" "robotics" "data science" "algorithms" "programming" "software engineering" "web development" "mobile development" "cloud computing" "cybersecurity" "blockchain" "quantum computing" "virtual reality" "augmented reality" "internet of things")

declare -a TECH_WORDS=("React" "Python" "JavaScript" "TypeScript" "Node.js" "Docker" "Kubernetes" "AWS" "Google Cloud" "Azure" "TensorFlow" "PyTorch" "OpenAI" "GPT" "BERT" "Transformer" "CNN" "RNN" "LSTM" "GAN")

declare -a BUSINESS_WORDS=("strategy" "innovation" "startup" "venture capital" "product management" "user experience" "design thinking" "agile" "scrum" "kanban" "team management" "leadership" "communication" "collaboration" "productivity" "efficiency" "automation" "optimization")

declare -a RESEARCH_WORDS=("analysis" "experiment" "hypothesis" "methodology" "results" "conclusion" "literature review" "survey" "case study" "benchmark" "evaluation" "comparison" "framework" "model" "algorithm" "implementation" "validation" "testing")

# Fonction pour générer du contenu aléatoire
generate_content() {
    local topic=$1
    local length=${2:-5}
    
    echo "# $topic"
    echo ""
    echo "Date: $(date +%Y-%m-%d)"
    echo "Tags: #$(echo $topic | tr ' ' '-'), #notes"
    echo ""
    
    # Paragraphe d'introduction
    echo "## Introduction"
    echo ""
    echo "This document explores the concepts and applications of $topic in modern technology."
    echo "It covers various aspects including theoretical foundations, practical implementations, and future directions."
    echo ""
    
    # Contenu principal avec mots-clés variés
    for i in $(seq 1 $length); do
        echo "## Section $i"
        echo ""
        
        # Mélanger différents types de mots
        case $((i % 4)) in
            0) words=("${TECH_WORDS[@]}");;
            1) words=("${BUSINESS_WORDS[@]}");;
            2) words=("${RESEARCH_WORDS[@]}");;
            3) words=("${TOPICS[@]}");;
        esac
        
        # Générer 2-3 phrases par section
        for j in $(seq 1 $((2 + RANDOM % 2))); do
            word1=${words[$RANDOM % ${#words[@]}]}
            word2=${words[$RANDOM % ${#words[@]}]}
            word3=${TECH_WORDS[$RANDOM % ${#TECH_WORDS[@]}]}
            
            echo "The integration of $word1 with $word2 represents a significant advancement in $word3 development."
        done
        echo ""
    done
    
    # Conclusion
    echo "## Conclusion"
    echo ""
    echo "The study of $topic continues to evolve with new methodologies and technologies."
    echo "Future research directions include enhanced automation, improved efficiency, and broader applications."
    echo ""
    echo "References: [1] Research Paper, [2] Technical Documentation, [3] Industry Report"
}

# Générer les notes dans différents dossiers
echo "📝 Génération des notes dans Projects..."
for i in $(seq 1 $((NOTES_COUNT / 5))); do
    topic=${TOPICS[$RANDOM % ${#TOPICS[@]}]}
    filename="$BASE_DIR/Notes/Projects/project_$(printf "%04d" $i)_$(echo $topic | tr ' ' '_' | tr '[:upper:]' '[:lower:]').md"
    generate_content "Project: $topic" $((3 + RANDOM % 4)) > "$filename"
done

echo "📝 Génération des notes dans Research..."
for i in $(seq 1 $((NOTES_COUNT / 5))); do
    topic=${RESEARCH_WORDS[$RANDOM % ${#RESEARCH_WORDS[@]}]}
    tech=${TECH_WORDS[$RANDOM % ${#TECH_WORDS[@]}]}
    filename="$BASE_DIR/Notes/Research/research_$(printf "%04d" $i)_${topic}_${tech}.md"
    generate_content "Research: $topic in $tech" $((4 + RANDOM % 3)) > "$filename"
done

echo "📝 Génération des notes dans Meetings..."
for i in $(seq 1 $((NOTES_COUNT / 5))); do
    date=$(date -v-${RANDOM}d +%Y%m%d)
    business=${BUSINESS_WORDS[$RANDOM % ${#BUSINESS_WORDS[@]}]}
    filename="$BASE_DIR/Notes/Meetings/meeting_${date}_${business}.md"
    generate_content "Meeting: $business Discussion" 2 > "$filename"
done

echo "📝 Génération des notes dans Daily..."
for i in $(seq 1 $((NOTES_COUNT / 5))); do
    date=$(date -v-${i}d +%Y-%m-%d)
    filename="$BASE_DIR/Notes/Daily/daily_${date}.md"
    generate_content "Daily Notes for $date" 3 > "$filename"
done

echo "📝 Génération des notes dans Archive..."
for i in $(seq 1 $((NOTES_COUNT / 5))); do
    topic=${TOPICS[$RANDOM % ${#TOPICS[@]}]}
    year=$((2020 + RANDOM % 4))
    filename="$BASE_DIR/Notes/Archive/archive_${year}_$(printf "%04d" $i)_$(echo $topic | tr ' ' '_').md"
    generate_content "Archived: $topic ($year)" $((2 + RANDOM % 3)) > "$filename"
done

# Générer quelques templates
echo "📝 Génération des templates..."
for template in "project" "meeting" "research" "daily" "review"; do
    filename="$BASE_DIR/Templates/${template}_template.md"
    generate_content "Template: $template" 3 > "$filename"
done

# Générer de la documentation
echo "📝 Génération de la documentation..."
for i in $(seq 1 20); do
    tech=${TECH_WORDS[$RANDOM % ${#TECH_WORDS[@]}]}
    filename="$BASE_DIR/Documentation/${tech}_documentation.md"
    generate_content "$tech Documentation" $((5 + RANDOM % 3)) > "$filename"
done

# Générer des idées
echo "📝 Génération des idées..."
for i in $(seq 1 30); do
    topic=${TOPICS[$RANDOM % ${#TOPICS[@]}]}
    business=${BUSINESS_WORDS[$RANDOM % ${#BUSINESS_WORDS[@]}]}
    filename="$BASE_DIR/Ideas/idea_$(printf "%03d" $i)_${topic// /_}_${business// /_}.md"
    generate_content "Idea: $topic for $business" 2 > "$filename"
done

# Compter le nombre total de fichiers générés
TOTAL_FILES=$(find "$BASE_DIR" -name "*.md" | wc -l | tr -d ' ')

echo ""
echo "✅ Corpus de test généré avec succès!"
echo "📊 Statistiques:"
echo "   📁 Répertoire: $BASE_DIR"
echo "   📄 Total de fichiers: $TOTAL_FILES"
echo "   💾 Taille totale: $(du -sh "$BASE_DIR" | cut -f1)"
echo ""
echo "🔍 Structure générée:"
find "$BASE_DIR" -type d | sort | sed 's/^/   /'
echo ""
echo "📈 Répartition par dossier:"
for dir in $(find "$BASE_DIR" -type d -not -path "$BASE_DIR"); do
    count=$(find "$dir" -name "*.md" | wc -l | tr -d ' ')
    echo "   $(basename "$dir"): $count fichiers"
done